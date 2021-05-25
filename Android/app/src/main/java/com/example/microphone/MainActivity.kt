package com.example.microphone

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.util.Log
import android.view.View
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import kotlinx.coroutines.*
import java.lang.Exception
import java.lang.IllegalArgumentException
import java.lang.Runnable

// helper function to ignore some exceptions
inline fun ignore(body: () -> Unit)
{
    try {
        body()
    }catch (e : Exception){}
}

class GlobalState(
    var isBluetoothStarted : Boolean,
    var bluetoothShouldStop : Boolean,
    var isAudioStarted : Boolean,
    var audioShouldStop : Boolean
)

class GlobalData()
{
    // TODO: design basic data structure
}

class MainActivity : AppCompatActivity()
{
    private val mLogTag : String = "MainActivityTag"
    private val mUIScope = CoroutineScope(Dispatchers.Main)
    private var mJobBluetooth : Job? = null

    private var mLogTextView : TextView? = null

    private var helperBluetooth : BluetoothHelper? = null
    private var helperAudio : AudioHelper? = null

    private val mGlobalState = GlobalState(
        false,
        false,
        false,
        false
    ) // global boolean shared by all threads

    private var threadBth : Thread? = null
    private var threadAio : Thread? = null

    override fun onCreate(savedInstanceState: Bundle?)
    {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        mLogTextView = findViewById(R.id.txt_log)
    }

    override fun onStop() {
        super.onStop()
        mGlobalState.bluetoothShouldStop = true
        mGlobalState.audioShouldStop = true
        threadBth?.join()
        threadAio?.join()
        helperBluetooth?.clean()

    }

    fun onButtonBluetooth(view : View)
    {
        if(!mGlobalState.isBluetoothStarted)
        {
            val activity = this
            if(mJobBluetooth?.isActive == true) return
            this.showToastMessage("Starting bluetooth...")
            Log.d(mLogTag, "onButtonBluetooth [start]")
            mJobBluetooth = mUIScope.launch {
                withContext(Dispatchers.Default)
                {
                    mGlobalState.bluetoothShouldStop = false
                    helperBluetooth?.clean()
                    helperBluetooth = try {
                        BluetoothHelper(activity)
                    } catch (e : IllegalArgumentException){
                        withContext(Dispatchers.Main)
                        {
                            activity.addLogMessage("Error: " + e.message)
                        }
                        null
                    }
                    if(helperBluetooth?.connect() == true)
                    {
                        withContext(Dispatchers.Main)
                        {
                            activity.showToastMessage("Device connected")
                            activity.addLogMessage("Connected Device Information\n${helperBluetooth?.getConnectedDeviceInfo()}\n")
                        }
                        Log.d(mLogTag, "onButtonBluetooth [connect]")
                    }
                    threadBth = Thread{
                        while (!mGlobalState.bluetoothShouldStop) {
                            if (helperBluetooth?.isSocketValid() == true) // check if socket is disconnected
                                helperBluetooth?.sendData()
                            else {
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
                this.addLogMessage("Device disconnected successfully\n")
            }
            helperBluetooth?.clean()
            helperBluetooth = null
            threadBth = null
            (view as Button).setText(R.string.turn_on_bth)
            mGlobalState.isBluetoothStarted = false
        }
    }

    private fun showToastMessage(message : String)
    {
        Toast.makeText(this, message, Toast.LENGTH_SHORT).show()
    }

    private fun addLogMessage(message : String)
    {
        mLogTextView?.append(message + "\n")
    }
}