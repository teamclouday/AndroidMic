package com.example.microphone

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.util.Log
import android.view.View
import android.view.WindowManager
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
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
    var audioShouldStop : Boolean
)

// global data structure
class GlobalData
{
    val MAX_BUFFER_SIZE = 30
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

    private lateinit var mLogTextView : TextView

    private var helperBluetooth : BluetoothHelper? = null
    private var helperAudio : AudioHelper? = null

    private val mGlobalState = GlobalState(
        false,
        false,
        false,
        false
    ) // global boolean shared by all threads

    private val mGlobalData = GlobalData()

    private var threadBth : Thread? = null
    private var threadAio : Thread? = null

    override fun onCreate(savedInstanceState: Bundle?)
    {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        mLogTextView = findViewById(R.id.txt_log)
        // set screen to always on
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
    }

    override fun onStop() {
        super.onStop()
        mGlobalState.bluetoothShouldStop = true
        mGlobalState.audioShouldStop = true
        threadBth?.join()
        threadAio?.join()
        helperBluetooth?.clean()
        helperAudio?.clean()
    }

    override fun onResume() {
        super.onResume()
        findViewById<Button>(R.id.bth_connect).setText(R.string.turn_on_bth)
        findViewById<SwitchCompat>(R.id.audio_switch).isChecked = false
        mGlobalState.isBluetoothStarted = false
        mGlobalState.isAudioStarted = false
    }

    override fun onPause() {
        super.onPause()
        mLogTextView.text = "" // clear log messages on pause
        mGlobalState.bluetoothShouldStop = true
        mGlobalState.audioShouldStop = true
        threadBth?.join()
        threadAio?.join()
        helperBluetooth?.clean()
        helperAudio?.clean()
    }

    // onclick for bluetooth button
    fun onButtonBluetooth(view : View)
    {
        if(!mGlobalState.isBluetoothStarted)
        {
            val activity = this
            if(mJobBluetooth?.isActive == true) return
            this.showToastMessage("Starting bluetooth...")
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
                        null
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
                    threadBth?.start()

                    // update UI if success start
                    if(helperBluetooth != null && threadBth?.isAlive == true)
                    {
                        withContext(Dispatchers.Main)
                        {
                            (view as Button).setText(R.string.turn_off_bth)
                        }
                        mGlobalState.isBluetoothStarted = true
                    }
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
            threadBth = null
            (view as Button).setText(R.string.turn_on_bth)
            mGlobalState.isBluetoothStarted = false
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
    fun showToastMessage(message : String)
    {
        Toast.makeText(this, message, Toast.LENGTH_SHORT).show()
    }

    // helper function to append log message to textview
    fun addLogMessage(message : String)
    {
        mLogTextView.append(message + "\n")
    }
}