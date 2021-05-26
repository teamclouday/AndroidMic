package com.example.microphone

import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothClass
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothSocket
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.util.Log
import androidx.core.content.ContextCompat
import android.Manifest
import android.content.pm.PackageManager
import java.io.*
import java.lang.Exception
import java.util.*

class BluetoothHelper(private val mActivity: MainActivity, private val mGlobalData : GlobalData)
{
    private val mLogTag : String = "AndroidMicBth"

    private val mUUID : UUID = UUID.fromString("34335e34-bccf-11eb-8529-0242ac130003")
    private val REQUEST_ENABLE_BIT = 1
    private val MAX_WAIT_TIME = 1500L // timeout

    private val DEVICE_CHECK_DATA : Int = 123456
    private val DEVICE_CHECK_EXPECTED : Int = 654321

    private val mAdapter : BluetoothAdapter? = BluetoothAdapter.getDefaultAdapter()
    private var mTargetDevice : BluetoothDevice? = null
    private var mSocket : BluetoothSocket? = null

    private val mReceiver = object : BroadcastReceiver()
    {
        override fun onReceive(context: Context?, intent: Intent?) {
            val action = intent?.action ?: return
            // check if server side is disconnected
            if(BluetoothAdapter.ACTION_STATE_CHANGED == action)
            {
                val state = intent.getIntExtra(BluetoothAdapter.EXTRA_STATE, BluetoothAdapter.ERROR)
                if(state == BluetoothAdapter.STATE_TURNING_OFF)
                    disconnect()
            }
            else if(BluetoothDevice.ACTION_ACL_DISCONNECTED == action)
                disconnect()
            else if(BluetoothDevice.ACTION_ACL_DISCONNECT_REQUESTED == action)
                disconnect()
        }
    }

    // init everything
    init
    {
        // check bluetooth adapter
        require(mAdapter != null) {"Bluetooth adapter is not found"}
        // check permission
        require(ContextCompat.checkSelfPermission(mActivity, Manifest.permission.BLUETOOTH) == PackageManager.PERMISSION_GRANTED){
            "Bluetooth is not permitted"
        }
        // enable adapter
        if(!mAdapter.isEnabled)
        {
            val enableBthIntent = Intent(BluetoothAdapter.ACTION_REQUEST_ENABLE)
            mActivity.startActivityForResult(enableBthIntent, REQUEST_ENABLE_BIT)
        }
        require(mAdapter.isEnabled){"Bluetooth adapter is not enabled"}
        // set target device
        selectTargetDevice()
        require(mTargetDevice != null) {"Cannot find target PC in paired device list"}
        // set up filters
        val filter = IntentFilter(BluetoothAdapter.ACTION_STATE_CHANGED)
        filter.addAction(BluetoothDevice.ACTION_ACL_DISCONNECT_REQUESTED)
        filter.addAction(BluetoothDevice.ACTION_ACL_DISCONNECTED)
        mActivity.registerReceiver(mReceiver, filter)
    }

    // connect to target device
    fun connect() : Boolean
    {
        // create socket
        val socket = try {
            mTargetDevice?.createInsecureRfcommSocketToServiceRecord(mUUID)
        } catch (e : IOException) {
            Log.d(mLogTag, "connect [createInsecureRfcommSocketToServiceRecord]: ${e.message}")
            null
        } ?: return false
        // connect to server
        try {
            socket.connect()
        } catch (e : IOException){
            Log.d(mLogTag, "connect [connect]: ${e.message}")
            return false
        }
        mSocket = socket
        Log.d(mLogTag, "connected")
        return true
    }

    // send data through socket
    fun sendData()
    {
        if(mSocket?.isConnected != true) return
        val nextData = mGlobalData.getData() ?: return
        try {
            val stream = mSocket?.outputStream
            stream?.write(nextData)
            stream?.flush()
            // Log.d(mLogTag, "[sendData] data sent (${nextData.size} bytes)")
        } catch (e : IOException)
        {
            Log.d(mLogTag, "${e.message}")
            Thread.sleep(4)
            disconnect()
        }
    }

    // disconnect from target device
    fun disconnect() : Boolean
    {
        if(mSocket == null) return false
        val socket = mSocket
        try {
            socket?.close()
        } catch(e : IOException) {
            Log.d(mLogTag, "disconnect [close]: ${e.message}")
            mSocket = null
            return false
        }
        mSocket = null
        Log.d(mLogTag, "disconnected")
        return true
    }

    // clean object
    fun clean()
    {
        disconnect()
        ignore { mActivity.unregisterReceiver(mReceiver) }
    }

    // auto select target PC device from a list
    private fun selectTargetDevice()
    {
        mTargetDevice = null
        val pairedDevices = mAdapter?.bondedDevices ?: return
        for(device in pairedDevices)
        {
            if(device.bluetoothClass.majorDeviceClass == BluetoothClass.Device.Major.COMPUTER)
                if(testConnection(device))
                {
                    mTargetDevice = device
                    break
                }
        }
    }

    // test connection with a device
    // return true if valid device
    // return false if invalid device
    private fun testConnection(device : BluetoothDevice) : Boolean
    {
        // get socket from device
        val socket : BluetoothSocket = try {
            device.createInsecureRfcommSocketToServiceRecord(mUUID)
        } catch (e : IOException) {
            Log.d(mLogTag, "testConnection [createInsecureRfcommSocketToServiceRecord]: ${e.message}")
            null
        } ?: return false
        // try to connect
        try {
            socket.connect()
        } catch (e : IOException){
            Log.d(mLogTag, "testConnection [connect]: ${e.message}")
            return false
        }

        var isValid = false

        val validationThread = Thread {
            // send a message to device and get response
            ignore {
                val streamOut = DataOutputStream(socket.outputStream)
                streamOut.writeInt(DEVICE_CHECK_DATA)
                streamOut.flush()
                val streamIn = DataInputStream(socket.inputStream)
                if(streamIn.readInt() == DEVICE_CHECK_EXPECTED)
                    isValid = true
            }
        }

        try {
            validationThread.start()
            validationThread.join(MAX_WAIT_TIME) // set max wait time for execution
            if(validationThread.isAlive)
            {
                Log.d(mLogTag, "testConnection [validationThread]: exceeds max timeout")
                ignore { socket.close() }
                return false
            }
        } catch(e : InterruptedException) {
            Log.d(mLogTag, "testConnection [validationThread]: exceeds max timeout $MAX_WAIT_TIME")
            ignore { socket.close() }
            return false
        } catch (e : Exception) {
            Log.d(mLogTag, "testConnection [validationThread]: ${e.message}")
            ignore { socket.close() }
            return false
        }
        // close socket
        ignore { socket.close() }
        return isValid
    }

    // get connected device information
    fun getConnectedDeviceInfo() : String
    {
        if(mAdapter == null || mTargetDevice == null || mSocket == null) return ""
        return "[Device Name] ${mTargetDevice?.name}\n[Device Address] ${mTargetDevice?.address}"
    }

    // check if current socket is valid and connected
    fun isSocketValid() : Boolean
    {
        return !(mSocket == null || mSocket?.isConnected == false)
    }
}