package com.example.microphone

import android.util.Log
import android.util.Patterns
import java.io.DataInputStream
import java.io.DataOutputStream
import java.io.EOFException
import java.io.IOException
import java.net.*


// enable connection through USB tethering
class USBHelper(private val mGlobalData : GlobalData)
{
    private val mLogTag : String = "AndroidMicUSB"

    private val MAX_WAIT_TIME = 1500 // timeout
    private val DEVICE_CHECK_DATA : Int = 123456
    private val DEVICE_CHECK_EXPECTED : Int = 654321
    private val PORT = 55555

    private var mSocket : Socket? = null
    private var mAddress : String = ""

    // init and check for USB tethering
    init
    {
        // reference: https://stackoverflow.com/questions/43478586/checking-tethering-usb-bluetooth-is-active
        // reference: https://airtower.wordpress.com/2010/07/29/getting-network-interface-information-in-java/
        val ifs = NetworkInterface.getNetworkInterfaces()
        var tetheringEnabled = false
        while(ifs.hasMoreElements())
        {
            val iface = ifs.nextElement()
            Log.d(mLogTag, "init checking iface = " + iface.name)
            if(iface.name == "rndis0" || iface.name == "ap0")
            {
                tetheringEnabled = true
                break
            }
        }
        require(tetheringEnabled){"USB tethering is not enabled"}

        // can also programmatically enable tethering according to this
        // but no idea how to stop tethering after program exit
        // reference: https://stackoverflow.com/questions/3436280/start-stop-built-in-wi-fi-usb-tethering-from-code
        // I prefer that the users can only enable it themselves
    }

    // connect to target device
    fun connect() : Boolean
    {
        // create socket and connect
        mSocket = Socket()
        try {
            mSocket?.connect(InetSocketAddress(mAddress, PORT), MAX_WAIT_TIME)
        } catch (e : IOException) {
            Log.d(mLogTag, "connect [Socket]: ${e.message}")
            null
        } catch (e : SocketTimeoutException) {
            Log.d(mLogTag, "connect [Socket]: ${e.message}")
            null
        } ?: return false
        mSocket?.soTimeout = MAX_WAIT_TIME
        // validate with server
        if(!validate())
        {
            mSocket?.close()
            mSocket = null
            return false
        }
        return true
    }

    // send data through socket
    fun sendData()
    {
        if(mSocket == null) return
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
        if(mSocket != null)
        {
            ignore { mSocket?.close() }
            mSocket = null
        }
        return true
    }

    // clean object
    fun clean()
    {
        disconnect()
    }

    // validate connection with the server
    private fun validate() : Boolean
    {
        if(!isSocketValid()) return false
        var isValid = false
        try {
            val streamOut = DataOutputStream(mSocket?.outputStream)
            streamOut.writeInt(DEVICE_CHECK_DATA)
            streamOut.flush()
            val streamIn = DataInputStream(mSocket?.inputStream)
            if(streamIn.readInt() == DEVICE_CHECK_EXPECTED)
                isValid = true
        } catch (e : EOFException) {
            Log.d(mLogTag, "validate error: ${e.message}")
            ignore { mSocket?.close() }
            mSocket = null
        } catch (e : IOException) {
            Log.d(mLogTag, "validate error: ${e.message}")
            ignore { mSocket?.close() }
            mSocket = null
        }
        return isValid
    }

    // set and validate IP address
    fun setAddress(address : String) : Boolean
    {
        mAddress = address
        Log.d(mLogTag, "setAddress ${address}")
        return Patterns.IP_ADDRESS.matcher(address).matches()
    }

    // get connected device information
    fun getConnectedDeviceInfo() : String
    {
        if(mSocket == null) return ""
        return "[Device Address] ${mSocket?.remoteSocketAddress}"
    }

    // check if current socket is valid and connected
    fun isSocketValid() : Boolean
    {
        return mSocket?.isConnected == true
    }
}