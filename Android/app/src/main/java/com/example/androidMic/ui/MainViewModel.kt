package com.example.androidMic.ui

import android.app.Application
import android.os.*
import android.util.Log
import android.widget.Toast
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.SavedStateHandle
import com.example.androidMic.AndroidMicApp
import com.example.androidMic.R
import com.example.androidMic.ui.utils.Preferences
import com.example.androidMic.domain.service.Command
import com.example.androidMic.domain.service.Command.Companion.COMMAND_DISC_STREAM
import com.example.androidMic.domain.service.Command.Companion.COMMAND_GET_STATUS
import com.example.androidMic.utils.checkIp
import com.example.androidMic.utils.checkPort


class MainViewModel(
    application: Application,
    private val savedStateHandle: SavedStateHandle
) : AndroidViewModel(application) {

    private val TAG = "MainViewModel"

    val uiStates = savedStateHandle.getStateFlow("uiStates", States.UiStates())

    private val preferences = Preferences(application as AndroidMicApp)

    private var mService: Messenger? = null
    private var mBound = false

    lateinit var handlerThread: HandlerThread
    private lateinit var mMessenger: Messenger
    lateinit var mMessengerLooper: Looper
    private lateinit var mMessengerHandler: ReplyHandler

    private inner class ReplyHandler(looper: Looper) : Handler(looper) {
        override fun handleMessage(msg: Message) {
            when (msg.what) {
                COMMAND_DISC_STREAM -> handleDisconnect(msg)
                COMMAND_GET_STATUS -> handleGetStatus(msg)

                else -> handleResult(msg)
            }
        }
    }

    fun handlerServiceResponse() {
        handlerThread = HandlerThread("activity", Process.THREAD_PRIORITY_BACKGROUND)
        handlerThread.start()
        mMessengerLooper = handlerThread.looper
        mMessengerHandler = ReplyHandler(mMessengerLooper)
        mMessenger = Messenger(mMessengerHandler)
    }

    init {
        Log.d(TAG, "init")
        val ipPort = preferences.getIpPort()
        val mode = preferences.getMode()
        val theme = preferences.getTheme()
        val dynamicColor = preferences.getDynamicColor()

        savedStateHandle["uiStates"] = uiStates.value.copy(
            ip = ipPort.first,
            port = ipPort.second,
            mode = mode,
            theme = theme,
            dynamicColor = dynamicColor
        )
    }

    fun refreshAppVariables() {
        mBound = getApplication<AndroidMicApp>().mBound
        mService = getApplication<AndroidMicApp>().mService

        savedStateHandle["uiStates"] = uiStates.value.copy(
            isAudioStarted = false,
            isStreamStarted = false,
            buttonConnectIsClickable = false,
            switchAudioIsClickable = false
        )
    }

    fun onConnectButton() {
        if (!mBound) return
        val reply = if (uiStates.value.isStreamStarted) {
            Log.d(TAG, "onConnectButton: stop stream")
            Message.obtain(null, Command.COMMAND_STOP_STREAM)
        } else {
            val data = Bundle()

            when (uiStates.value.mode) {
                Modes.WIFI, Modes.UDP -> {
                    if (!checkIp(uiStates.value.ip) || !checkPort(uiStates.value.port)) {
                        Toast.makeText(
                            getApplication(),
                            getApplication<AndroidMicApp>().getString(R.string.invalid_ip_port),
                            Toast.LENGTH_SHORT
                        ).show()
                        savedStateHandle["uiStates"] = uiStates.value.copy(
                            dialogVisible = Dialogs.IpPort
                        )
                        return
                    }
                    data.putString("IP", uiStates.value.ip)
                    data.putInt("PORT", uiStates.value.port.toInt())
                }
                Modes.USB -> {
                    if (!checkPort(uiStates.value.port)) {
                        Toast.makeText(
                            getApplication(),
                            getApplication<AndroidMicApp>().getString(R.string.invalid_port),
                            Toast.LENGTH_SHORT
                        ).show()
                        savedStateHandle["uiStates"] = uiStates.value.copy(
                            dialogVisible = Dialogs.IpPort
                        )
                        return
                    }
                    data.putInt("PORT", uiStates.value.port.toInt())
                }
                else -> { }
            }

            data.putInt("MODE", uiStates.value.mode.ordinal)

            Log.d(TAG, "onConnectButton: start stream")
            // lock button to avoid duplicate events
            savedStateHandle["uiStates"] = uiStates.value.copy(
                buttonConnectIsClickable = false
            )
            Message.obtain(null, Command.COMMAND_START_STREAM).apply {
                this.data = data
            }
        }

        reply.replyTo = mMessenger
        mService?.send(reply)
    }

    fun onAudioSwitch() {
        if (!mBound) return
        val reply = if (uiStates.value.isAudioStarted) {
            Log.d(TAG, "onAudioSwitch: stop audio")
            Message.obtain(null, Command.COMMAND_STOP_AUDIO)
        } else {
            Log.d(TAG, "onAudioSwitch: start audio")
            Message.obtain(null, Command.COMMAND_START_AUDIO)
        }
        // lock switch to avoid duplicate events
        savedStateHandle["uiStates"] = uiStates.value.copy(
            switchAudioIsClickable = false
        )
        reply.replyTo = mMessenger
        mService?.send(reply)
    }

    fun setIpPort(ip: String, port: String) {
        preferences.setIpPort(ip, port)
        savedStateHandle["uiStates"] = uiStates.value.copy(
            ip = ip,
            port = port,
            dialogVisible = Dialogs.None
        )
    }

    fun setMode(mode: Modes) {
        preferences.setMode(mode)
        savedStateHandle["uiStates"] = uiStates.value.copy(
            mode = mode,
            dialogVisible = Dialogs.None
        )
    }

    fun showDialog(dialog: Dialogs) {
        savedStateHandle["uiStates"] = uiStates.value.copy(dialogVisible = dialog)
    }

    fun setTheme(theme: Themes) {
        preferences.setTheme(theme)
        savedStateHandle["uiStates"] =
            uiStates.value.copy(
                theme = theme
            )
    }

    fun setDynamicColor(dynamicColor: Boolean) {
        preferences.setDynamicColor(dynamicColor)
        savedStateHandle["uiStates"] =
            uiStates.value.copy(
                dynamicColor = dynamicColor
            )
    }

    fun cleanLog() {
        savedStateHandle["uiStates"] =
            uiStates.value.copy(
                textLog = ""
            )
    }


    // ask foreground service for current status
    fun askForStatus() {
        if (!mBound) return
        val reply = Message.obtain(null, COMMAND_GET_STATUS)
        reply.replyTo = mMessenger
        mService?.send(reply)
    }


    // apply status to UI
    private fun handleGetStatus(msg: Message) {
        savedStateHandle["uiStates"] = uiStates.value.copy(
            isStreamStarted = msg.data.getBoolean("isStreamStarted"),
            isAudioStarted = msg.data.getBoolean("isAudioStarted"),
            switchAudioIsClickable = true,
            buttonConnectIsClickable = true
        )
    }

    private fun handleResult(msg: Message) {
        val reply = msg.data.getString("reply")
        if (reply != null) addLogMessage(reply)

        val result = msg.data.getBoolean("result")

        when (msg.what) {
            Command.COMMAND_START_STREAM -> savedStateHandle["uiStates"] = uiStates.value.copy(
                isStreamStarted = result,
                buttonConnectIsClickable = true
            )
            Command.COMMAND_STOP_STREAM -> savedStateHandle["uiStates"] = uiStates.value.copy(
                isStreamStarted = !result,
                buttonConnectIsClickable = true
            )

            Command.COMMAND_START_AUDIO -> savedStateHandle["uiStates"] = uiStates.value.copy(
                isAudioStarted = result,
                switchAudioIsClickable = true
            )
            Command.COMMAND_STOP_AUDIO -> savedStateHandle["uiStates"] = uiStates.value.copy(
                isAudioStarted = !result,
                switchAudioIsClickable = true
            )
        }
    }

    private fun handleDisconnect(msg: Message) {
        Log.d(TAG, "handleDisconnect")
        val reply = msg.data.getString("reply")
        if (reply != null) addLogMessage(reply)
        savedStateHandle["uiStates"] = uiStates.value.copy(
            isStreamStarted = false
        )
    }

    // helper function to append log message to textview
    private fun addLogMessage(message: String) {
        savedStateHandle["uiStates"] = uiStates.value.copy(
            textLog = uiStates.value.textLog + message + "\n"
        )
    }
}