package io.github.teamclouday.AndroidMic.domain.streaming

import Message
import android.Manifest
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothClass
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothSocket
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageManager
import android.os.Build
import android.os.Messenger
import android.util.Log
import androidx.core.content.ContextCompat
import com.google.protobuf.ByteString
import io.github.teamclouday.AndroidMic.domain.service.AudioPacket
import io.github.teamclouday.AndroidMic.utils.chunked
import io.github.teamclouday.AndroidMic.utils.ignore
import io.github.teamclouday.AndroidMic.utils.toBigEndianU32
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import java.io.IOException
import java.util.UUID

class BluetoothStreamer(private val ctx: Context, val scope: CoroutineScope) : Streamer {
    private val TAG: String = "MicStreamBTH"

    private val myUUID: UUID = UUID.fromString("34335e34-bccf-11eb-8529-0242ac130003")

    private val adapter: BluetoothAdapter
    private var target: BluetoothDevice? = null
    private var socket: BluetoothSocket? = null
    private var streamJob: Job? = null

    private val receiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context?, intent: Intent?) {
            val action = intent?.action ?: return
            // check if server side is disconnected
            if (BluetoothAdapter.ACTION_STATE_CHANGED == action) {
                val state = intent.getIntExtra(BluetoothAdapter.EXTRA_STATE, BluetoothAdapter.ERROR)
                if (state == BluetoothAdapter.STATE_TURNING_OFF)
                    disconnect()
            } else if (BluetoothDevice.ACTION_ACL_DISCONNECTED == action)
                disconnect()
            else if (BluetoothDevice.ACTION_ACL_DISCONNECT_REQUESTED == action)
                disconnect()
        }
    }

    // init everything
    init {
        // get bluetooth adapter
        val bm = ctx.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
        adapter = bm.adapter
        // check permission
        require(
            ContextCompat.checkSelfPermission(
                ctx,
                Manifest.permission.BLUETOOTH
            ) == PackageManager.PERMISSION_GRANTED
        ) {
            "Bluetooth is not permitted"
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            require(
                ContextCompat.checkSelfPermission(
                    ctx,
                    Manifest.permission.BLUETOOTH_CONNECT
                ) == PackageManager.PERMISSION_GRANTED
            ) {
                "Bluetooth is not permitted"
            }
        }
        require(adapter.isEnabled) { "Bluetooth adapter is not enabled" }
        // set target device
        selectTargetDevice()
        require(target != null) { "Cannot find target PC in paired device list" }
        // set up filters
        val filter = IntentFilter(BluetoothAdapter.ACTION_STATE_CHANGED)
        filter.addAction(BluetoothDevice.ACTION_ACL_DISCONNECT_REQUESTED)
        filter.addAction(BluetoothDevice.ACTION_ACL_DISCONNECTED)
        ctx.registerReceiver(receiver, filter)
    }

    // connect to target device
    override fun connect(): Boolean {
        // create socket
        socket = try {
            target?.createInsecureRfcommSocketToServiceRecord(myUUID)
        } catch (e: IOException) {
            Log.d(TAG, "connect [createInsecureRfcommSocketToServiceRecord]: ${e.message}")
            null
        } catch (e: SecurityException) {
            Log.d(TAG, "connect [createInsecureRfcommSocketToServiceRecord]: ${e.message}")
            null
        } ?: return false
        // connect to server
        try {
            socket?.connect()
        } catch (e: IOException) {
            Log.d(TAG, "connect [connect]: ${e.message}")
            return false
        } catch (e: SecurityException) {
            Log.d(TAG, "connect [connect]: ${e.message}")
            return false
        }
        Log.d(TAG, "connect: connected")
        return true
    }

    // stream data through socket
    override fun start(audioStream: Flow<AudioPacket>, tx: Messenger) {
        streamJob?.cancel()

        streamJob = scope.launch {
            audioStream.collect { data ->
                if (socket == null || socket?.isConnected != true) return@collect

                try {
                    val message = Message.AudioPacketMessage.newBuilder()
                        .setBuffer(ByteString.copyFrom(data.buffer))
                        .setSampleRate(data.sampleRate)
                        .setAudioFormat(data.audioFormat)
                        .setChannelCount(data.channelCount)
                        .build()
                    val pack = message.toByteArray()

                    socket!!.outputStream.write(pack.size.toBigEndianU32())

                    for (chunk in pack.chunked(1024)) {
                        socket!!.outputStream.write(chunk)
                    }

                    socket!!.outputStream.flush()
                } catch (e: IOException) {
                    Log.d(TAG, "stream: ${e.message}")
                    delay(5)
                    disconnect()
                } catch (e: Exception) {
                    Log.d(TAG, "stream: ${e.message}")
                }
            }
        }
    }

    // disconnect from target device
    override fun disconnect(): Boolean {
        if (socket == null) return false
        try {
            socket?.close()
        } catch (e: IOException) {
            Log.d(TAG, "disconnect [close]: ${e.message}")
            socket = null
            return false
        }
        socket = null
        streamJob?.cancel()
        streamJob = null
        Log.d(TAG, "disconnect: complete")
        return true
    }

    // shutdown streamer
    override fun shutdown() {
        disconnect()
        ignore { ctx.unregisterReceiver(receiver) }
    }

    // auto select target PC device from a bounded devices list
    private fun selectTargetDevice() {
        target = null
        try {
            val pairedDevices = adapter.bondedDevices ?: return
            for (device in pairedDevices) {
                if (device.bluetoothClass.majorDeviceClass == BluetoothClass.Device.Major.COMPUTER) {
                    target = device
                    Log.d(TAG, "selected device ${device.name}")
                    break
                }
            }
        } catch (e: SecurityException) {
            Log.d(TAG, "selectTargetDevice: ${e.message}")
        }

    }

    // get connected device information
    override fun getInfo(): String {
        if (target == null || socket == null) return ""
        val deviceName = try {
            target?.name
        } catch (e: SecurityException) {
            Log.d(TAG, "getInfo: ${e.message}")
            "null"
        }
        return "[Device Name]:${deviceName}\n[Device Address]:${target?.address}"
    }

    // return true if is connected for streaming
    override fun isAlive(): Boolean {
        return (socket != null && socket?.isConnected == true)
    }
}