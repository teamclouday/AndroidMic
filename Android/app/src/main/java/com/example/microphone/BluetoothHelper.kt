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
import java.io.*
import java.lang.Exception
import java.util.*

class BluetoothHelper(private val mActivity: MainActivity)
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

    init
    {
        // check bluetooth adapter
        require(mAdapter != null) {"Bluetooth adapter is not found"}
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
    public fun connect() : Boolean
    {
        val socket = try {
            mTargetDevice?.createInsecureRfcommSocketToServiceRecord(mUUID)
        } catch (e : IOException) {
            Log.d(mLogTag, "connect [createInsecureRfcommSocketToServiceRecord]: ${e.message}")
            null
        } ?: return false
        try {
            socket.connect()
        } catch (e : IOException){
            Log.d(mLogTag, "connect [connect]: ${e.message}")
            return false
        }
        mSocket = socket
        return true
    }

    // send data through socket
    public fun sendData()
    {
        // TODO: collect data from variable and send data to target device
    }

    // disconnect from target device
    public fun disconnect() : Boolean
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
        return true
    }

    // clean object
    public fun clean()
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
                val streamIn = DataInputStream(socket.inputStream)
                if(streamIn.readInt() == DEVICE_CHECK_EXPECTED)
                    isValid = true
            }
        }

        try {
            validationThread.start()
            validationThread.join(MAX_WAIT_TIME)
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

        ignore { socket.close() }
        return isValid
    }

    // get connected device information
    public fun getConnectedDeviceInfo() : String
    {
        if(mAdapter == null || mTargetDevice == null || mSocket == null) return ""
        return "[Device Name] ${mTargetDevice?.name}\n[Device Address] ${mTargetDevice?.address}"
    }

    public fun isSocketValid() : Boolean
    {
        return !(mSocket == null || mSocket?.isConnected == false)
    }
}