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
import com.example.androidMic.utils.Command
import com.example.androidMic.utils.Command.Companion.COMMAND_DISC_STREAM
import com.example.androidMic.utils.Command.Companion.COMMAND_GET_STATUS
import com.example.androidMic.utils.CommandService
import com.example.androidMic.utils.Modes.Companion.MODE_USB
import com.example.androidMic.utils.Modes.Companion.MODE_WIFI
import com.example.androidMic.utils.States


class MainViewModel(
    application: Application,
    private val savedStateHandle: SavedStateHandle
) : AndroidViewModel(application) {

    private val TAG = "MainViewModel"

    val uiStates = savedStateHandle.getStateFlow("uiStates", States.UiStates())

    private val preferences = Preferences(application as AndroidMicApp)

    private var mService: Messenger? = null
    var mBound = false

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
        val ipPort = preferences.getWifiIpPort(true)
        val usbPort = preferences.getUsbPort()
        val mode = preferences.getMode()
        val theme = preferences.getTheme()
        val dynamicColor = preferences.getDynamicColor()

        savedStateHandle["uiStates"] = uiStates.value.copy(
            IP = ipPort.first,
            PORT = ipPort.second.toString(),
            usbPort = usbPort.toString(),
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

    fun onEvent(event: Event) {
        var reply: Message? = null
        when (event) {
            is Event.ConnectButton -> {
                if (!mBound) return
                if (uiStates.value.isStreamStarted) {
                    Log.d(TAG, "onConnectButton: stop stream")
                    reply = Message.obtain(null, Command.COMMAND_STOP_STREAM)
                } else {
                    val data = Bundle()
                    if (uiStates.value.mode == MODE_WIFI) {
                        try {
                            val (ip, port) = preferences.getWifiIpPort(false)
                            data.putString("IP", ip)
                            data.putInt("PORT", port)
                        } catch (e: Exception) {
                            Toast.makeText(
                                getApplication(),
                                getApplication<AndroidMicApp>().getString(R.string.invalid_ip_port),
                                Toast.LENGTH_SHORT
                            ).show()
                            savedStateHandle["uiStates"] = uiStates.value.copy(
                                dialogIpPortIsVisible = true
                            )
                            return
                        }
                    }
                    if (uiStates.value.mode == MODE_USB) {
                        val port = preferences.getUsbPort()
                        data.putInt("PORT", port)
                    }
                    data.putInt("MODE", uiStates.value.mode)
                    reply = Message.obtain(null, Command.COMMAND_START_STREAM)
                    reply.data = data
                    Log.d(TAG, "onConnectButton: start stream")
                    // lock button to avoid duplicate events
                    savedStateHandle["uiStates"] = uiStates.value.copy(
                        buttonConnectIsClickable = false
                    )
                }
            }

            is Event.AudioSwitch -> {
                if (!mBound) return
                reply = if (uiStates.value.isAudioStarted) {
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
            }

            is Event.SetWifiIpPort -> {
                try {
                    preferences.setWifiIpPort(Pair(event.ip, event.port))
                } catch (e: Exception) {
                    Toast.makeText(
                        getApplication(),
                        getApplication<AndroidMicApp>().getString(R.string.invalid_ip_port),
                        Toast.LENGTH_SHORT
                    ).show()
                    return
                }
                savedStateHandle["uiStates"] = uiStates.value.copy(
                    IP = event.ip,
                    PORT = event.port,
                    dialogIpPortIsVisible = false
                )
            }

            is Event.SetUsbPort -> {
                try {
                    preferences.setUsbPort(event.port)
                } catch (e: Exception) {
                    Toast.makeText(
                        getApplication(),
                        getApplication<AndroidMicApp>().getString(R.string.invalid_port),
                        Toast.LENGTH_SHORT
                    ).show()
                    return
                }


                savedStateHandle["uiStates"] = uiStates.value.copy(
                    usbPort = event.port,
                    dialogUsbPortIsVisible = false
                )
            }
            is Event.SetMode -> {
                preferences.setMode(event.mode)
                savedStateHandle["uiStates"] = uiStates.value.copy(
                    mode = event.mode,
                    dialogModesIsVisible = false
                )
            }

            is Event.ShowDialog -> {
                when (event.id) {
                    R.string.drawerWifiIpPort -> savedStateHandle["uiStates"] =
                        uiStates.value.copy(dialogIpPortIsVisible = true)
                    R.string.drawerUsbPort -> savedStateHandle["uiStates"] =
                        uiStates.value.copy(dialogUsbPortIsVisible = true)
                    R.string.drawerMode -> savedStateHandle["uiStates"] =
                        uiStates.value.copy(dialogModesIsVisible = true)
                    R.string.drawerTheme -> savedStateHandle["uiStates"] =
                        uiStates.value.copy(dialogThemeIsVisible = true)
                }
            }
            is Event.DismissDialog -> {
                when (event.id) {
                    R.string.drawerWifiIpPort -> savedStateHandle["uiStates"] =
                        uiStates.value.copy(dialogIpPortIsVisible = false)
                    R.string.drawerUsbPort -> savedStateHandle["uiStates"] =
                        uiStates.value.copy(dialogUsbPortIsVisible = false)
                    R.string.drawerMode -> savedStateHandle["uiStates"] =
                        uiStates.value.copy(dialogModesIsVisible = false)
                    R.string.drawerTheme -> savedStateHandle["uiStates"] =
                        uiStates.value.copy(dialogThemeIsVisible = false)
                }
            }

            is Event.SetTheme -> {
                preferences.setTheme(event.theme)
                savedStateHandle["uiStates"] =
                    uiStates.value.copy(
                        theme = event.theme
                    )
            }

            is Event.SetDynamicColor -> {
                preferences.setDynamicColor(event.dynamicColor)
                savedStateHandle["uiStates"] =
                    uiStates.value.copy(
                        dynamicColor = event.dynamicColor
                    )
            }

            is Event.CleanLog -> {
                savedStateHandle["uiStates"] =
                    uiStates.value.copy(
                        textLog = ""
                    )
            }
        }
        if (reply != null) {
            reply.replyTo = mMessenger
            mService?.send(reply)
        }
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

        val log = if (result) "handleSuccess" else "handleFailure"
        // for log
        val commandService = CommandService()
        Log.d(TAG, "$log: ${commandService.dic[msg.what]}")
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