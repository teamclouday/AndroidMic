package com.example.microphone

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.text.InputType
import android.util.Log
import android.view.View
import android.view.WindowManager
import android.widget.*
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.widget.SwitchCompat
import kotlinx.coroutines.*
import java.lang.Exception
import java.lang.IllegalArgumentException

// helper function to ignore some exceptions
inline fun ignore(body: () -> Unit)
{
    try {
        body()
    }catch (e : Exception){}
}

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
    val MAX_BUFFER_SIZE = 5 // set to 5 to reduce latency
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

class MainActivity : AppCompatActivity()
{
    private val mLogTag : String = "MainActivityTag"
    private val mUIScope = CoroutineScope(Dispatchers.Main)
    private var mJobBluetooth : Job? = null
    private var mJobAudio : Job? = null
    private var mJobUSB : Job? = null

    private lateinit var mLogTextView : TextView
    private lateinit var mScroller : ScrollView

    private var helperBluetooth : BluetoothHelper? = null
    private var helperAudio : AudioHelper? = null
    private var helperUSB : USBHelper? = null

    private val mGlobalState = GlobalState(
        false,
        false,
        false,
        false,
        false,
        false,
        false
    ) // global boolean shared by all threads

    private val mGlobalData = GlobalData()

    private var threadBth : Thread? = null
    private var threadAio : Thread? = null
    private var threadUSB : Thread? = null

    override fun onCreate(savedInstanceState: Bundle?)
    {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        mLogTextView = findViewById(R.id.txt_log)
        // set screen to always on
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        // set scroll to focus down
        mScroller = findViewById(R.id.scrollView2)
    }

    override fun onStop() {
        super.onStop()
        clean()
    }

    override fun onResume() {
        super.onResume()
        findViewById<Button>(R.id.bth_connect).setText(R.string.turn_on_bth)
        findViewById<Button>(R.id.usb_connect).setText(R.string.turn_on_usb)
        findViewById<SwitchCompat>(R.id.audio_switch).isChecked = false
        mGlobalState.isBluetoothStarted = false
        mGlobalState.isAudioStarted = false
        mGlobalState.isUSBStarted = false
    }

    override fun onPause() {
        super.onPause()
        mLogTextView.text = "" // clear log messages on pause
        clean()
    }

    fun clean()
    {
        mGlobalState.bluetoothShouldStop = true
        mGlobalState.audioShouldStop = true
        mGlobalState.usbShouldStop = true
        threadBth?.join()
        threadAio?.join()
        threadUSB?.join()
        helperBluetooth?.clean()
        helperAudio?.clean()
        helperUSB?.clean()
        if(mJobUSB?.isActive == true) mJobUSB?.cancel()
        if(mJobBluetooth?.isActive == true) mJobBluetooth?.cancel()
        if(mJobAudio?.isActive == true) mJobAudio?.cancel()
    }

    // onclick for bluetooth button
    fun onButtonBluetooth(view : View)
    {
        if(!mGlobalState.isBluetoothStarted)
        {
            if(isConnected())
            {
                showToastMessage("Already connected")
                return
            }
            val activity = this
            if(mJobBluetooth?.isActive == true) return
            Log.d(mLogTag, "onButtonBluetooth [start]")
            mGlobalData.reset() // reset global data to store most recent audio
            // launch a coroutine to prepare bluetooth helper object and start thread
            mJobBluetooth = mUIScope.launch {
                withContext(Dispatchers.Default)
                {
                    mGlobalState.bluetoothShouldStop = false
                    helperBluetooth?.clean()
                    // create object
                    helperBluetooth = try {
                        BluetoothHelper(activity, mGlobalData)
                    } catch (e : IllegalArgumentException){
                        withContext(Dispatchers.Main)
                        {
                            activity.addLogMessage("Error: " + e.message)
                        }
                        cancel()
                        null
                    }
                    withContext(Dispatchers.Main)
                    {
                        activity.showToastMessage("Starting bluetooth...")
                    }
                    // try to connect
                    if(helperBluetooth?.connect() == true)
                    {
                        withContext(Dispatchers.Main)
                        {
                            activity.showToastMessage("Device connected")
                            activity.addLogMessage("Connected Device Information\n${helperBluetooth?.getConnectedDeviceInfo()}")
                        }
                        Log.d(mLogTag, "onButtonBluetooth [connect]")
                    }
                    // define and start the thread
                    threadBth = Thread{
                        while (!mGlobalState.bluetoothShouldStop) {
                            if (helperBluetooth?.isSocketValid() == true) // check if socket is disconnected
                                helperBluetooth?.sendData()
                            else {
                                // if not valid, disconnect the device
                                runOnUiThread {
                                    activity.addLogMessage("Device disconnected")
                                    mGlobalState.isBluetoothStarted = false
                                    helperBluetooth?.clean()
                                    helperBluetooth = null
                                    threadBth = null
                                    (view as Button).setText(R.string.turn_on_bth)
                                }
                                break
                            }
                            Thread.sleep(1)
                        }
                    }
                    // update UI if success start
                    if(helperBluetooth?.isSocketValid() == true)
                    {
                        withContext(Dispatchers.Main)
                        {
                            (view as Button).setText(R.string.turn_off_bth)
                        }
                        mGlobalState.isBluetoothStarted = true
                    }

                    threadBth?.start()
                }
            }
        }
        else
        {
            mGlobalState.bluetoothShouldStop = true
            this.showToastMessage("Stopping bluetooth...")
            Log.d(mLogTag, "onButtonBluetooth [stop]")
            ignore { threadBth?.join(1000) }
            threadBth = null
            if(helperBluetooth?.disconnect() == true)
            {
                this.showToastMessage("Device disconnected")
                this.addLogMessage("Device disconnected successfully")
            }
            helperBluetooth?.clean()
            helperBluetooth = null
            (view as Button).setText(R.string.turn_on_bth)
            mGlobalState.isBluetoothStarted = false
        }
    }

    // onclick for USB button
    fun onButtonUSB(view : View)
    {
        if(!mGlobalState.isUSBStarted)
        {
            if(isConnected())
            {
                showToastMessage("Already connected")
                return
            }
            val activity = this
            if(mJobUSB?.isActive == true) return
            Log.d(mLogTag, "onButtonUSB [start]")
            mGlobalData.reset() // reset global data to store most recent audio
            // launch a coroutine to prepare bluetooth helper object and start thread
            mJobUSB = mUIScope.launch {
                withContext(Dispatchers.Default)
                {
                    mGlobalState.usbShouldStop = false
                    helperUSB?.clean()
                    // create object
                    helperUSB = try {
                        USBHelper(activity, mGlobalData)
                    } catch (e : IllegalArgumentException){
                        withContext(Dispatchers.Main)
                        {
                            activity.addLogMessage("Error: " + e.message)
                        }
                        cancel()
                        null
                    }
                    // get target IP address
                    withContext(Dispatchers.Main)
                    {
                        getIpAddress()
                    }
                    while(!mGlobalState.usbShouldStop && !mGlobalState.isUSBAddressSet)
                    {
                        try {
                            delay(200)
                        } catch (e : CancellationException) {break}
                    }
                    withContext(Dispatchers.Main)
                    {
                        activity.showToastMessage("Starting USB client...")
                    }
                    // try to connect
                    if(helperUSB?.connect() == true)
                    {
                        withContext(Dispatchers.Main)
                        {
                            activity.showToastMessage("Device connected")
                            activity.addLogMessage("Connected Device Information\n${helperUSB?.getConnectedDeviceInfo()}")
                        }
                        Log.d(mLogTag, "onButtonUSB [connect]")
                    }
                    // define and start the thread
                    threadUSB = Thread{
                        while (!mGlobalState.usbShouldStop) {
                            if (helperUSB?.isSocketValid() == true) // check if socket is disconnected
                                helperUSB?.sendData()
                            else {
                                // if not valid, disconnect the device
                                runOnUiThread {
                                    activity.addLogMessage("Device disconnected")
                                    mGlobalState.isUSBStarted = false
                                    helperUSB?.clean()
                                    helperUSB = null
                                    threadUSB = null
                                    (view as Button).setText(R.string.turn_on_usb)
                                }
                                break
                            }
                            Thread.sleep(1)
                        }
                    }
                    // update UI if success start
                    if(helperUSB?.isSocketValid() == true)
                    {
                        withContext(Dispatchers.Main)
                        {
                            (view as Button).setText(R.string.turn_off_usb)
                        }
                        mGlobalState.isUSBStarted = true
                    }

                    threadUSB?.start()
                }
            }
        }
        else
        {
            mGlobalState.usbShouldStop = true
            this.showToastMessage("Stopping USB client...")
            Log.d(mLogTag, "onButtonUSB [stop]")
            ignore { threadUSB?.join(1000) }
            threadUSB = null
            if(helperUSB?.disconnect() == true)
            {
                this.showToastMessage("Device disconnected")
                this.addLogMessage("Device disconnected successfully")
            }
            helperUSB?.clean()
            helperUSB = null
            (view as Button).setText(R.string.turn_on_usb)
            mGlobalState.isUSBStarted = false
        }
    }

    // on change for microphone switch
    // basically similar to bluetooth function
    fun onSwitchMic(view : View)
    {
        if(!mGlobalState.isAudioStarted)
        {
            val activity = this
            if(mJobAudio?.isActive == true) return
            this.showToastMessage("Starting microphone...")
            Log.d(mLogTag, "onSwitchMic [start]")
            mJobAudio = mUIScope.launch {
                withContext(Dispatchers.Default)
                {
                    mGlobalState.audioShouldStop = false
                    helperAudio?.clean()
                    helperAudio = try {
                        AudioHelper(activity, mGlobalData)
                    } catch (e : IllegalArgumentException){
                        withContext(Dispatchers.Main)
                        {
                            activity.addLogMessage("Error: " + e.message)
                        }
                        null
                    }
                    if(helperAudio?.startMic() != null)
                    {
                        withContext(Dispatchers.Main)
                        {
                            activity.showToastMessage("recording started")
                            activity.addLogMessage("Microphone starts recording")
                        }
                        Log.d(mLogTag, "onSwitchMic [recording]")
                    }
                    threadAio = Thread{
                        while (!mGlobalState.audioShouldStop) {
                            helperAudio?.setData() // set recorded audio data
                            Thread.sleep(1)
                        }
                    }
                    threadAio?.start()

                    if(helperAudio != null && threadAio?.isAlive == true)
                    {
                        withContext(Dispatchers.Main)
                        {
                            (view as SwitchCompat).isChecked = true
                        }
                        mGlobalState.isAudioStarted = true
                    }
                    else
                    {
                        withContext(Dispatchers.Main)
                        {
                            (view as SwitchCompat).isChecked = false
                        }
                    }
                }
            }
        }
        else
        {
            mGlobalState.audioShouldStop = true
            Log.d(mLogTag, "onSwitchMic [stop]")
            ignore { threadAio?.join(1000) }
            threadAio = null
            if(helperAudio?.stopMic() != null)
            {
                this.showToastMessage("recording stopped")
                this.addLogMessage("Microphone stops recording")
            }
            helperAudio?.clean()
            helperAudio = null
            threadAio = null
            (view as SwitchCompat).isChecked = false
            mGlobalState.isAudioStarted = false
        }
    }

    // helper function to show toast message
    private fun showToastMessage(message : String)
    {
        Toast.makeText(this, message, Toast.LENGTH_SHORT).show()
    }

    // helper function to append log message to textview
    private fun addLogMessage(message : String)
    {
        mLogTextView.append(message + "\n")
        mScroller.fullScroll(View.FOCUS_DOWN)
    }

    private fun isConnected() : Boolean
    {
        return mGlobalState.isUSBStarted || mGlobalState.isBluetoothStarted
    }

    private fun getIpAddress()
    {
        mGlobalState.isUSBAddressSet = false
        val builder =  AlertDialog.Builder(this)
        builder.setTitle("PC IP Address")
        val input = EditText(this)
        input.setText("192.168.")
        input.inputType = InputType.TYPE_CLASS_PHONE
        builder.setView(input)
        builder.setPositiveButton("OK"
        ) { dialog, which ->
            if(helperUSB?.setAddress(input.text.toString()) != true)
                addLogMessage("Invalid address: ${input.text}")
            mGlobalState.isUSBAddressSet = true
        }
        builder.setOnCancelListener {
            mGlobalState.isUSBAddressSet = true
        }
        builder.setOnDismissListener {
            mGlobalState.isUSBAddressSet = true
        }
        builder.show()
    }
}