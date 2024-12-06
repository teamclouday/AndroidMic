package com.example.androidMic.domain.streaming

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.hardware.usb.UsbAccessory
import android.hardware.usb.UsbManager
import android.util.Log
import com.example.androidMic.domain.service.AudioPacket
import com.example.androidMic.utils.toBigEndianU32
import com.google.protobuf.ByteString
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import java.io.FileOutputStream

class UsbStreamer(ctx: Context, private val scope: CoroutineScope) : Streamer {

    private val TAG: String = "USB streamer"

    private var streamJob: Job? = null

    private var accessory: UsbAccessory? = null
    private var outputStream: FileOutputStream? = null

    private val receiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context?, intent: Intent?) {
            val action = intent?.action ?: return

//            if (action == UsbManager.ACTION_USB_ACCESSORY_ATTACHED) {
//                Log.d(TAG, "onReceive: USB accessory attached")
//
//                outputStream?.close()
//                accessory = IntentCompat.getParcelableExtra<UsbAccessory>(
//                    intent,
//                    UsbManager.EXTRA_ACCESSORY,
//                    UsbAccessory::class.java
//                )
//
//                val fd = (ctx.getSystemService(Context.USB_SERVICE) as UsbManager).openAccessory(
//                    accessory
//                ).fileDescriptor
//                outputStream = FileOutputStream(fd)
//            }
        }
    }

    // init everything
    init {
        // set up filter
        val filter = IntentFilter(UsbManager.ACTION_USB_ACCESSORY_ATTACHED)
        ctx.registerReceiver(receiver, filter)

        // select usb device
        val usbManager = ctx.getSystemService(Context.USB_SERVICE) as UsbManager
        val accessoryList = usbManager.accessoryList

        require(!accessoryList.isNullOrEmpty()) {
            "No USB device detected"
        }

        accessory = accessoryList[0];

        Log.d(
            TAG,
            "init USB accessory: ${accessory?.manufacturer} ${accessory?.model} ${accessory?.version}"
        )

        // open stream
        val fd = usbManager.openAccessory(accessory).fileDescriptor
        require(fd != null) {
            "Failed to open USB accessory file descriptor"
        }
        outputStream = FileOutputStream(fd)
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

    override fun start(audioStream: Flow<AudioPacket>) {
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