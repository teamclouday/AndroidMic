package com.example.microphone

import android.app.Service
import android.content.Intent
import android.os.*
import android.util.Log
import android.widget.Toast
import kotlinx.coroutines.*
import java.lang.IllegalArgumentException

// global state struct for managing threads
class GlobalState(
    var isBluetoothStarted : Boolean,
    var bluetoothShouldStop : Boolean,
    var isAudioStarted : Boolean,
    var audioShouldStop : Boolean,
    var isUSBStarted : Boolean,
    var usbShouldStop : Boolean,
    var isUSBAddressSet : Boolean
)

// global data structure
class GlobalData
{
    private val MAX_BUFFER_SIZE = 5 // set to 5 to reduce latency
    private val mBuffer = mutableListOf<ByteArray>()
    private val mLock = object{}

    fun addData(data : ByteArray)
    {
        synchronized(mLock)
        {
            mBuffer.add(data)
            while(mBuffer.size > MAX_BUFFER_SIZE)
                mBuffer.removeFirst()
        }
    }

    fun getData() : ByteArray?
    {
        synchronized(mLock)
        {
            return mBuffer.removeFirstOrNull()
        }
    }

    fun reset()
    {
        synchronized(mLock)
        {
            mBuffer.clear()
        }
    }
}

// reference : https://stackoverflow.com/questions/9489075/android-service-activity-2-way-communication

class BackgroundHelper : Service()
{
    private val mLogTag = "BackgroundHelper"
    private val mUIScope = CoroutineScope(Dispatchers.Main)
    private var mJobBluetooth : Job? = null
    private var mJobAudio : Job? = null
    private var mJobUSB : Job? = null

    private lateinit var handlerThread : HandlerThread
    private lateinit var serviceLooper : Looper
    private lateinit var serviceHandler : ServiceHandler
    private lateinit var serviceMessenger : Messenger

    companion object
    {
        const val COMMAND_START_BLUETOOTH = 1
        const val COMMAND_STOP_BLUETOOTH = 2
        const val COMMAND_START_AUDIO = 3
        const val COMMAND_STOP_AUDIO = 4
        const val COMMAND_START_USB = 5
        const val COMMAND_STOP_USB = 6

        const val COMMAND_DISC_BTH = 20
        const val COMMAND_DISC_USB = 21

        const val COMMAND_SET_IP = 30
    }

    private var helperBluetooth : BluetoothHelper? = null
    private var helperAudio : AudioHelper? = null
    private var helperUSB : USBHelper? = null

    private val mGlobalState = GlobalState(
        isBluetoothStarted = false,
        bluetoothShouldStop = false,
        isAudioStarted = false,
        audioShouldStop = false,
        isUSBStarted = false,
        usbShouldStop = false,
        isUSBAddressSet = false
    ) // global boolean shared by all threads

    private val mGlobalData = GlobalData()

    private var threadBth : Thread? = null
    private var threadUSB : Thread? = null
    private var threadAio : Thread? = null


    private inner class ServiceHandler(looper: Looper) : Handler(looper)
    {
        override fun handleMessage(msg: Message)
        {
            when(msg.what)
            {
                COMMAND_START_BLUETOOTH -> startBluetooth(msg)
                COMMAND_STOP_BLUETOOTH  -> stopBluetooth(msg)
                COMMAND_START_AUDIO     -> startAudio(msg)
                COMMAND_STOP_AUDIO      -> stopAudio(msg)
                COMMAND_START_USB       -> startUSB(msg)
                COMMAND_STOP_USB        -> stopUSB(msg)
                COMMAND_SET_IP          -> setIP(msg)
            }
        }
    }

    override fun onCreate()
    {
        handlerThread = HandlerThread("BackgroundHelperStartArgs", Process.THREAD_PRIORITY_BACKGROUND)
        handlerThread.start()
        serviceLooper = handlerThread.looper
        serviceHandler = ServiceHandler(handlerThread.looper)
        serviceMessenger = Messenger(serviceHandler)
    }

    override fun onBind(intent: Intent?): IBinder?
    {
        return serviceMessenger.binder
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int
    {
        return START_STICKY
    }

    override fun onDestroy()
    {
        if(mJobUSB?.isActive == true) mJobUSB?.cancel()
        if(mJobBluetooth?.isActive == true) mJobBluetooth?.cancel()
        if(mJobAudio?.isActive == true) mJobAudio?.cancel()
        mGlobalState.bluetoothShouldStop = true
        mGlobalState.audioShouldStop = true
        mGlobalState.usbShouldStop = true
        threadBth?.join()
        threadAio?.join()
        threadUSB?.join()
        helperBluetooth?.clean()
        helperAudio?.clean()
        helperUSB?.clean()
        serviceLooper.quitSafely()
        ignore { handlerThread.join(1000) }
    }

    fun startBluetooth(msg : Message)
    {
        val sender = msg.replyTo
        val replyData = Bundle()
        if(!mGlobalState.isBluetoothStarted)
        {
            Log.d(mLogTag, "startBluetooth [start]")
            if(isConnected())
            {
                showToastMessage("Already connected")
                replyFailed(sender, replyData, COMMAND_START_BLUETOOTH)
            }
            else
            {
                if(mJobBluetooth?.isActive == true)
                {
                    replyFailed(sender, replyData, COMMAND_START_BLUETOOTH)
                    return
                }
                mGlobalData.reset()
                mJobBluetooth = mUIScope.launch {
                    withContext(Dispatchers.Default) {
                        helperBluetooth?.clean()
                        helperBluetooth = try {
                            BluetoothHelper(applicationContext, mGlobalData)
                        } catch (e: IllegalArgumentException) {
                            replyData.putString("reply", "Error: " + e.message)
                            replyFailed(sender, replyData, COMMAND_START_BLUETOOTH)
                            cancel()
                            null
                        }
                        withContext(Dispatchers.Main) {
                            showToastMessage("Starting bluetooth...")
                        }
                        if (helperBluetooth?.connect() == true) {
                            withContext(Dispatchers.Main) {
                                showToastMessage("Device connected")
                            }
                            replyData.putString(
                                "reply",
                                "Connected Device Information\n${helperBluetooth?.getConnectedDeviceInfo()}"
                            )
                            Log.d(mLogTag, "startBluetooth [connected]")
                        }
                        else
                        {
                            replyData.putString("reply", "Failed to connect to bluetooth server")
                            replyFailed(sender, replyData, COMMAND_START_BLUETOOTH)
                            cancel()
                        }
                        mGlobalState.bluetoothShouldStop = false
                        threadBth = Thread {
                            while (!mGlobalState.bluetoothShouldStop) {
                                if (helperBluetooth?.isSocketValid() == true) // check if socket is disconnected
                                    helperBluetooth?.sendData()
                                else {
                                    val data = Bundle()
                                    replyData.putString("reply", "Device disconnected")
                                    serviceHandler.post {
                                        showToastMessage("Device disconnected")
                                    }
                                    val reply = Message()
                                    reply.data = data
                                    reply.what = COMMAND_DISC_BTH
                                    reply.replyTo = serviceMessenger
                                    sender.send(reply)
                                    mGlobalState.isBluetoothStarted = false
                                    helperBluetooth?.clean()
                                    helperBluetooth = null
                                    break
                                }
                                Thread.sleep(1)
                            }
                        }
                        if (helperBluetooth?.isSocketValid() == true) {
                            replySuccess(sender, replyData, COMMAND_START_BLUETOOTH)
                            mGlobalState.isBluetoothStarted = true
                        }
                        threadBth?.start()
                    }
                }
            }
        }
        else
        {
            replyData.putString("reply", "Bluetooth already started")
            replyFailed(sender, replyData, COMMAND_START_BLUETOOTH)
        }
    }

    fun stopBluetooth(msg : Message)
    {
        val sender = msg.replyTo
        Log.d(mLogTag, "stopBluetooth")
        val replyData = Bundle()
        if(mJobBluetooth?.isActive == true)
        {
            replyFailed(sender, replyData, COMMAND_STOP_BLUETOOTH)
            return
        }
        showToastMessage("Stopping bluetooth...")
        ignore { threadBth?.join(1000) }
        threadBth = null
        if(helperBluetooth?.disconnect() == true)
            replyData.putString("reply", "Device disconnected successfully")
        showToastMessage("Device disconnected")
        helperBluetooth?.clean()
        helperBluetooth = null
        mGlobalState.isBluetoothStarted = false
        replySuccess(sender, replyData, COMMAND_STOP_BLUETOOTH)
    }

    fun startAudio(msg : Message)
    {
        val sender = msg.replyTo
        val replyData = Bundle()
        if(!mGlobalState.isAudioStarted)
        {
            Log.d(mLogTag, "startAudio [start]")
            if(mJobAudio?.isActive == true)
            {
                replyFailed(sender, replyData, COMMAND_START_AUDIO)
                return
            }
            mJobAudio = mUIScope.launch {
                withContext(Dispatchers.Default) {
                    helperAudio?.clean()
                    helperAudio = try {
                        AudioHelper(applicationContext, mGlobalData)
                    } catch (e: IllegalArgumentException) {
                        replyData.putString("reply", "Error: " + e.message)
                        replyFailed(sender, replyData, COMMAND_START_AUDIO)
                        cancel()
                        null
                    }
                    if (helperAudio?.startMic() != null) {
                        withContext(Dispatchers.Main) {
                            showToastMessage("Recording started")
                        }
                        replyData.putString("reply", "Microphone starts recording")
                        Log.d(mLogTag, "startAudio [recording]")
                    }
                    else cancel()
                    mGlobalState.audioShouldStop = false
                    threadAio = Thread {
                        while (!mGlobalState.audioShouldStop) {
                            helperAudio?.setData()
                            Thread.sleep(1)
                        }
                    }
                    replySuccess(sender, replyData, COMMAND_START_AUDIO)
                    mGlobalState.isAudioStarted = true
                    threadAio?.start()
                }
            }
        }
        else
        {
            replyData.putString("reply", "Microphone already enabled")
            replyFailed(sender, replyData, COMMAND_START_AUDIO)
        }
    }

    fun stopAudio(msg : Message)
    {
        val sender = msg.replyTo
        Log.d(mLogTag, "stopAudio")
        val replyData = Bundle()
        if(mJobAudio?.isActive == true)
        {
            replyFailed(sender, replyData, COMMAND_STOP_AUDIO)
            return
        }
        ignore { threadAio?.join(1000) }
        threadAio = null
        if(helperAudio?.stopMic() != null)
            replyData.putString("reply", "Microphone stops recording")
        showToastMessage("Recording stopped")
        helperAudio?.clean()
        helperAudio = null
        mGlobalState.isAudioStarted = false
        replySuccess(sender, replyData, COMMAND_STOP_AUDIO)
    }

    fun startUSB(msg : Message)
    {
        val sender = msg.replyTo
        val replyData = Bundle()
        if(!mGlobalState.isUSBStarted)
        {
            Log.d(mLogTag, "startUSB [start]")
            if(isConnected())
            {
                showToastMessage("Already connected")
                replyFailed(sender, replyData, COMMAND_START_USB)
            }
            else
            {
                if(mJobUSB?.isActive == true)
                {
                    replyFailed(sender, replyData, COMMAND_START_USB)
                    return
                }
                mGlobalData.reset()
                mJobUSB = mUIScope.launch {
                    withContext(Dispatchers.Default) {
                        helperUSB?.clean()
                        helperUSB = try {
                            USBHelper(mGlobalData)
                        } catch (e: IllegalArgumentException) {
                            replyData.putString("reply", "Error: " + e.message)
                            replyFailed(sender, replyData, COMMAND_START_USB)
                            cancel()
                            null
                        }
                        withContext(Dispatchers.Main) {
                            showToastMessage("Starting USB client...")
                        }
                        // ask to get IP
                        mGlobalState.isUSBAddressSet = false
                        val askIP = Message()
                        askIP.data = null
                        askIP.what = COMMAND_SET_IP
                        askIP.replyTo = serviceMessenger
                        sender.send(askIP)
                        // wait for IP
                        while (!mGlobalState.usbShouldStop && !mGlobalState.isUSBAddressSet) {
                            try {
                                delay(200)
                            } catch (e: CancellationException) {
                                break
                            }
                        }
                        if (helperUSB?.connect() == true) {
                            withContext(Dispatchers.Main) {
                                showToastMessage("Device connected")
                            }
                            replyData.putString(
                                "reply",
                                "Connected Device Information\n${helperUSB?.getConnectedDeviceInfo()}"
                            )
                            Log.d(mLogTag, "startUSB [connected]")
                        }
                        else
                        {
                            replyData.putString("reply", "Failed to connect to USB server")
                            replyFailed(sender, replyData, COMMAND_START_USB)
                            cancel()
                        }
                        mGlobalState.usbShouldStop = false
                        threadUSB = Thread {
                            while (!mGlobalState.usbShouldStop) {
                                if (helperUSB?.isSocketValid() == true) // check if socket is disconnected
                                    helperUSB?.sendData()
                                else {
                                    val data = Bundle()
                                    replyData.putString("reply", "Device disconnected")
                                    serviceHandler.post{
                                        showToastMessage("Device disconnected")
                                    }
                                    val reply = Message()
                                    reply.data = data
                                    reply.what = COMMAND_DISC_USB
                                    reply.replyTo = serviceMessenger
                                    sender.send(reply)
                                    mGlobalState.isUSBStarted = false
                                    helperUSB?.clean()
                                    helperUSB = null
                                    break
                                }
                                Thread.sleep(1)
                            }
                        }
                        if (helperUSB?.isSocketValid() == true) {
                            replySuccess(sender, replyData, COMMAND_START_USB)
                            mGlobalState.isUSBStarted = true
                        }
                        threadUSB?.start()
                    }
                }
            }
        }
        else
        {
            replyData.putString("reply", "USB client already started")
            replyFailed(sender, replyData, COMMAND_START_USB)
        }
    }

    fun stopUSB(msg : Message)
    {
        val sender = msg.replyTo
        Log.d(mLogTag, "stopUSB")
        val replyData = Bundle()
        if(mJobUSB?.isActive == true)
        {
            replyFailed(sender, replyData, COMMAND_STOP_USB)
            return
        }
        showToastMessage("Stopping USB client...")
        ignore { threadUSB?.join(1000) }
        threadUSB = null
        if(helperUSB?.disconnect() == true)
            replyData.putString("reply", "Device disconnected successfully")
        showToastMessage("Device disconnected")
        helperUSB?.clean()
        helperUSB = null
        mGlobalState.isUSBStarted = false
        replySuccess(sender, replyData, COMMAND_STOP_USB)
    }

    fun setIP(msg : Message)
    {
        val sender = msg.replyTo
        val replyData = Bundle()
        val ip = msg.data.getString("IP") ?: ""
        mGlobalState.isUSBAddressSet = true
        if(helperUSB?.setAddress(ip) != true)
        {
            replyData.putString("reply", "Invalid address: $ip")
            replyFailed(sender, replyData, COMMAND_SET_IP)
        }
        else replySuccess(sender, replyData, COMMAND_SET_IP)
    }

    private fun isConnected() : Boolean
    {
        return mGlobalState.isBluetoothStarted || mGlobalState.isUSBStarted
    }

    private fun replyFailed(sender : Messenger, data : Bundle, what : Int)
    {
        data.putInt("Result", 0) // 0 for failure
        val reply = Message()
        reply.data = data
        reply.what = what
        reply.replyTo = serviceMessenger
        sender.send(reply)
    }

    private fun replySuccess(sender : Messenger, data : Bundle, what : Int)
    {
        data.putInt("Result", 1) // 1 for success
        val reply = Message()
        reply.data = data
        reply.what = what
        reply.replyTo = serviceMessenger
        sender.send(reply)
    }

    private fun showToastMessage(message : String)
    {
        Toast.makeText(this, message, Toast.LENGTH_SHORT).show()
    }
}