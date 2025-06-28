package io.github.teamclouday.AndroidMic.domain.streaming

import Message
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
import io.github.teamclouday.AndroidMic.domain.service.AudioPacket
import io.github.teamclouday.AndroidMic.utils.toBigEndianU32
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import java.io.FileDescriptor
import java.io.FileInputStream
import java.io.FileOutputStream

class UsbStreamer(ctx: Context, private val scope: CoroutineScope) : Streamer {

    private val TAG: String = "USB streamer"
    private val USB_PERMISSION = "io.github.teamclouday.AndroidMic.USB_PERMISSION"

    private var streamJob: Job? = null

    private var accessory: UsbAccessory? = null
    private var accessoryPfd: ParcelFileDescriptor? = null
    private var accessoryFd: FileDescriptor? = null
    private var outputStream: FileOutputStream? = null
    private var inputStream: FileInputStream? = null

    private val receiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context?, intent: Intent?) {
            val action = intent?.action ?: return

            if (action == UsbManager.ACTION_USB_ACCESSORY_DETACHED) {
                Log.d(TAG, "onReceive: USB accessory detached")

                val acc = BundleCompat.getParcelable<UsbAccessory>(
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

                    val pfd = usbManager.openAccessory(accessory)
                    val fd = pfd?.fileDescriptor

                    if (fd == null) {
                        Log.d(TAG, "Failed to open USB accessory file descriptor")
                        return
                    }
                    accessoryPfd = pfd
                    accessoryFd = fd
                    outputStream = FileOutputStream(fd)
                    inputStream = FileInputStream(fd)
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

        accessory = accessoryList[0];

        Log.d(
            TAG,
            "choose USB accessory: ${accessory?.manufacturer} ${accessory?.model} ${accessory?.version}"
        )

        // check permission
        if (!usbManager.hasPermission(accessory)) {
            Log.d(TAG, "requesting permission")
            usbManager.requestPermission(
                accessory, PendingIntent.getBroadcast(
                    ctx, 0, Intent(USB_PERMISSION), 0
                )
            )
        }

        require(usbManager.hasPermission(accessory)) {
            "Failed to get permission for USB accessory"
        }

        // open stream
        val pfd = usbManager.openAccessory(accessory)
        val fd = pfd?.fileDescriptor
        require(fd != null) {
            "Failed to open USB accessory file descriptor"
        }
        accessoryPfd = pfd
        accessoryFd = fd
        outputStream = FileOutputStream(fd)
        inputStream = FileInputStream(fd)
    }

    override fun connect(): Boolean {
        return true
    }

    override fun disconnect(): Boolean {
        streamJob?.cancel()
        streamJob = null
        Log.d(TAG, "disconnect: complete")
        return true
    }

    override fun shutdown() {
        outputStream?.close()
        disconnect()
    }

    override fun start(audioStream: Flow<AudioPacket>, tx: Messenger) {
        streamJob?.cancel()

        streamJob = scope.launch {
            audioStream.collect { data ->
                if (accessory == null || outputStream == null) return@collect

                try {
                    val message = Message.AudioPacketMessage.newBuilder()
                        .setBuffer(ByteString.copyFrom(data.buffer))
                        .setSampleRate(data.sampleRate)
                        .setAudioFormat(data.audioFormat)
                        .setChannelCount(data.channelCount)
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