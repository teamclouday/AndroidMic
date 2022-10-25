package com.example.androidMic.domain.streaming

import android.Manifest
import android.bluetooth.*
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageManager
import android.os.Build
import android.util.Log
import androidx.core.content.ContextCompat
import com.example.androidMic.domain.audio.AudioBuffer
import com.example.androidMic.utils.ignore
import kotlinx.coroutines.*
import java.io.DataInputStream
import java.io.DataOutputStream
import java.io.IOException
import java.util.*

class BluetoothStreamer(private val ctx: Context) : Streamer {
    private val TAG: String = "MicStreamBTH"

    private val myUUID: UUID = UUID.fromString("34335e34-bccf-11eb-8529-0242ac130003")
    private val MAX_WAIT_TIME = 1500L // timeout

    private val adapter: BluetoothAdapter
    private var target: BluetoothDevice? = null
    private var socket: BluetoothSocket? = null

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
        // check bluetooth adapter
        require(adapter != null) { "Bluetooth adapter is not found" }
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
    override suspend fun stream(audioBuffer: AudioBuffer) = withContext(Dispatchers.IO)
    {
        if (socket == null || socket?.isConnected != true || audioBuffer.isEmpty()) return@withContext
        var readSize = 0
        try {
            val streamOut = socket!!.outputStream
            val region = audioBuffer.openReadRegion(Streamer.BUFFER_SIZE)
            val regionSize = region.first
            val regionOffset = region.second
            streamOut.write(audioBuffer.buffer, regionOffset, regionSize)
            readSize = regionSize
            // streamOut.flush()
        } catch (e: IOException) {
            Log.d(TAG, "stream: ${e.message}")
            delay(5)
            disconnect()
            readSize = 0
        } catch (e: Exception) {
            Log.d(TAG, "stream: ${e.message}")
            readSize = 0
        } finally {
            audioBuffer.closeReadRegion(readSize)
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
                    Log.d(TAG, "selectTargetDevice: testing ${device.name}")
                    if (testConnection(device)) {
                        target = device
                        Log.d(TAG, "selectTargetDevice: ${device.name} is valid")
                        break
                    } else
                        Log.d(TAG, "selectTargetDevice: ${device.name} is invalid")
                }
            }
        } catch (e: SecurityException) {
            Log.d(TAG, "selectTargetDevice: ${e.message}")
        }

    }

    // test connection with a device
    // return true if valid device
    // return false if invalid device
    private fun testConnection(device: BluetoothDevice): Boolean {
        // get socket from device
        val socket: BluetoothSocket = try {
            device.createInsecureRfcommSocketToServiceRecord(myUUID)
        } catch (e: IOException) {
            Log.d(TAG, "testConnection [createInsecureRfcommSocketToServiceRecord]: ${e.message}")
            null
        } catch (e: SecurityException) {
            Log.d(TAG, "testConnection [createInsecureRfcommSocketToServiceRecord]: ${e.message}")
            null
        } ?: return false
        // try to connect
        try {
            socket.connect()
        } catch (e: IOException) {
            Log.d(TAG, "testConnection [connect]: ${e.message}")
            return false
        } catch (e: SecurityException) {
            Log.d(TAG, "testConnection [connect]: ${e.message}")
            return false
        }
        var isValid = false
        runBlocking(Dispatchers.IO) {
            val job = launch {
                ignore {
                    val streamOut = DataOutputStream(socket.outputStream)
                    streamOut.write(Streamer.DEVICE_CHECK.toByteArray(Charsets.UTF_8))
                    streamOut.flush()
                    val streamIn = DataInputStream(socket.inputStream)
                    val buffer = ByteArray(100)
                    val size = streamIn.read(buffer, 0, 100)
                    val received = String(buffer, 0, size, Charsets.UTF_8)
                    Log.d(TAG, "testConnection: received $received")
                    if (received == Streamer.DEVICE_CHECK_EXPECT) {
                        isValid = true
                        Log.d(TAG, "testConnection: device matched!")
                    } else
                        Log.d(TAG, "testConnection: device mismatch with $received!")
                }
            }
            var time = 5
            while (!job.isCompleted && time < MAX_WAIT_TIME) {
                delay(5)
                time += 5
            }
            if (!job.isCompleted) {
                ignore { socket.close() }
                job.cancelAndJoin()
                Log.d(TAG, "testConnection: timeout!")
            }
        }
        // close socket
        ignore { socket.close() }
        return isValid
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
        return "[Device Name] ${deviceName}\n[Device Address] ${target?.address}"
    }

    // return true if is connected for streaming
    override fun isAlive(): Boolean {
        return (socket != null && socket?.isConnected == true)
    }
}