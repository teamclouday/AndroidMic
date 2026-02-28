package io.github.teamclouday.androidMic.domain.streaming

import Message.Messages
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.hardware.usb.UsbAccessory
import android.hardware.usb.UsbManager
import android.os.Build
import android.os.Messenger
import android.os.ParcelFileDescriptor
import android.util.Log
import androidx.core.os.BundleCompat
import com.google.protobuf.ByteString
import io.github.teamclouday.androidMic.domain.service.AudioPacket
import io.github.teamclouday.androidMic.utils.toBigEndianU32
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.runInterruptible
import kotlinx.coroutines.withTimeout
import java.io.FileDescriptor
import java.io.FileInputStream
import java.io.FileOutputStream


private const val TAG: String = "USB streamer"

class UsbStreamer(ctx: Context, private val scope: CoroutineScope) : Streamer {

    companion object {
        private const val USB_PERMISSION = "io.github.teamclouday.AndroidMic.USB_PERMISSION"
    }

    private var streamJob: Job? = null

    private var accessory: UsbAccessory? = null
    private var accessoryPfd: ParcelFileDescriptor? = null
    private var accessoryFd: FileDescriptor? = null
    private var outputStream: FileOutputStream? = null
    private var inputStream: FileInputStream? = null
    private var sequenceIdx = 0

    private val receiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context?, intent: Intent?) {
            val action = intent?.action ?: return

            if (action == UsbManager.ACTION_USB_ACCESSORY_DETACHED) {
                Log.d(TAG, "onReceive: USB accessory detached")

                val acc = BundleCompat.getParcelable(
                    intent.extras!!,
                    UsbManager.EXTRA_ACCESSORY,
                    UsbAccessory::class.java,
                )

                if (acc == accessory) {
                    shutdown()
                }
            } else if (action == USB_PERMISSION) {
                val granted = intent.getBooleanExtra(UsbManager.EXTRA_PERMISSION_GRANTED, false)
                if (granted) {
                    Log.d(TAG, "permission granted")
                    val usbManager = ctx.getSystemService(Context.USB_SERVICE) as UsbManager
                    openAccessory(usbManager)
                } else {
                    Log.d(TAG, "permission denied")
                }
            }
        }
    }

    // init everything
    init {
        // set up filter
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            val filter = IntentFilter(
                UsbManager.ACTION_USB_ACCESSORY_DETACHED
            )
            filter.addAction(USB_PERMISSION)
            ctx.registerReceiver(receiver, filter, Context.RECEIVER_NOT_EXPORTED)
        } else {
            val filter = IntentFilter(
                UsbManager.ACTION_USB_ACCESSORY_DETACHED
            )
            ctx.registerReceiver(receiver, filter)
        }

        // select usb device
        val usbManager = ctx.getSystemService(Context.USB_SERVICE) as UsbManager
        val accessoryList = usbManager.accessoryList

        require(!accessoryList.isNullOrEmpty()) {
            "No USB device detected"
        }

        accessory = accessoryList[0]

        Log.d(
            TAG,
            "choose USB accessory: ${accessory?.manufacturer} ${accessory?.model} ${accessory?.version}"
        )

        // check permission and open accessory if already granted
        if (!usbManager.hasPermission(accessory)) {
            Log.d(TAG, "requesting permission")
            usbManager.requestPermission(
                accessory, PendingIntent.getBroadcast(
                    ctx, 0, Intent(USB_PERMISSION), PendingIntent.FLAG_IMMUTABLE
                )
            )
        } else {
            // Permission already granted, open accessory immediately
            Log.d(TAG, "permission already granted")
            openAccessory(usbManager)
        }
    }

    private fun openAccessory(usbManager: UsbManager) {
        val pfd = usbManager.openAccessory(accessory)
        val fd = pfd?.fileDescriptor

        if (fd == null) {
            Log.e(TAG, "Failed to open USB accessory file descriptor")
            return
        }

        accessoryPfd = pfd
        accessoryFd = fd
        outputStream = FileOutputStream(fd)
        inputStream = FileInputStream(fd)
        Log.d(TAG, "USB accessory opened successfully")
    }

    override fun connect(): Boolean {
        val outStream = outputStream
        val inStream = inputStream

        if (outStream == null || inStream == null) {
            Log.e(TAG, "connect: streams not initialized")
            return false
        }

        return try {
            // Send connect message
            val message = Messages.MessageWrapper.newBuilder()
                .setConnect(
                    Messages.ConnectMessage.newBuilder()
                        .build()
                )
                .build()

            val pack = message.toByteArray()

            // Write size header and message
            outStream.write(pack.size.toBigEndianU32())
            outStream.write(pack)
            outStream.flush()

            Log.d(TAG, "connect: sent connect message")

            // Wait for response with timeout using coroutines
            val buff = ByteArray(CHECK_2.length)

            val success = runBlocking {
                try {
                    withTimeout(1500L) {
                        runInterruptible {
                            var totalBytesRead = 0
                            while (totalBytesRead < buff.size) {
                                val bytesRead = inStream.read(
                                    buff,
                                    totalBytesRead,
                                    buff.size - totalBytesRead
                                )
                                Log.d(TAG, "connect: read $bytesRead bytes from input stream")

                                if (bytesRead > 0) {
                                    totalBytesRead += bytesRead
                                } else if (bytesRead == -1) {
                                    throw Exception("Stream closed")
                                }
                            }

                            buff.contentEquals(CHECK_2.toByteArray())
                        }
                    }
                } catch (_: TimeoutCancellationException) {
                    Log.e(TAG, "connect: timeout waiting for response")
                    false
                } catch (e: Exception) {
                    Log.e(TAG, "connect: ${e.message}", e)
                    false
                }
            }

            Log.d(TAG, "connect: handshake ${if (success) "successful" else "failed"}")
            success
        } catch (e: Exception) {
            Log.e(TAG, "connect: ${e.message}", e)
            false
        }
    }

    override fun disconnect(): Boolean {
        streamJob?.cancel()
        streamJob = null
        Log.d(TAG, "disconnect: complete")
        return true
    }

    override fun shutdown() {
        try {
            inputStream?.close()
            inputStream = null

            outputStream?.close()
            outputStream = null

            accessoryPfd?.close()
            accessoryPfd = null

            accessoryFd = null
        } catch (e: Exception) {
            Log.e(TAG, "shutdown: ${e.message}")
        }

        disconnect()
    }

    override fun start(audioStream: Flow<AudioPacket>, tx: Messenger) {
        streamJob?.cancel()

        streamJob = scope.launch {
            audioStream.collect { data ->
                if (accessory == null || outputStream == null) return@collect

                try {
                    val message = Messages.MessageWrapper.newBuilder()
                        .setAudioPacket(
                            Messages.AudioPacketMessageOrdered.newBuilder()
                                .setSequenceNumber(sequenceIdx++)
                                .setAudioPacket(
                                    Messages.AudioPacketMessage.newBuilder()
                                        .setBuffer(ByteString.copyFrom(data.buffer))
                                        .setSampleRate(data.sampleRate)
                                        .setAudioFormat(data.audioFormat)
                                        .setChannelCount(data.channelCount)
                                )
                                .build()
                        )
                        .build()

                    val pack = message.toByteArray()

//                    Log.d(TAG, "usb stream: sending ${pack.size} bytes")

                    outputStream!!.write(pack.size.toBigEndianU32())
                    outputStream!!.write(pack)
                    outputStream!!.flush()
                } catch (e: Exception) {
                    Log.d(TAG, "stream: ${e.message}")
                }
            }
        }
    }

    override fun getInfo(): String {
        if (accessory == null) return "No USB device detected"
        return "[USB Accessory Model]:${accessory?.model}\n[Manufacturer]:${accessory?.manufacturer}\n[Version]:${accessory?.version}"
    }

    override fun isAlive(): Boolean {
        return true
    }


}