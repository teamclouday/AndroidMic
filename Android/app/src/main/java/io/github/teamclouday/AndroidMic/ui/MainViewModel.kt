package io.github.teamclouday.AndroidMic.ui

import android.os.Handler
import android.os.HandlerThread
import android.os.Looper
import android.os.Message
import android.os.Messenger
import android.os.Process
import android.util.Log
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import io.github.teamclouday.AndroidMic.AndroidMicApp
import io.github.teamclouday.AndroidMic.AppPreferences
import io.github.teamclouday.AndroidMic.AudioFormat
import io.github.teamclouday.AndroidMic.ChannelCount
import io.github.teamclouday.AndroidMic.Dialogs
import io.github.teamclouday.AndroidMic.Mode
import io.github.teamclouday.AndroidMic.R
import io.github.teamclouday.AndroidMic.SampleRates
import io.github.teamclouday.AndroidMic.Themes
import io.github.teamclouday.AndroidMic.domain.service.Command
import io.github.teamclouday.AndroidMic.domain.service.CommandData
import io.github.teamclouday.AndroidMic.domain.service.ResponseKind
import io.github.teamclouday.AndroidMic.domain.service.ResponseData
import io.github.teamclouday.AndroidMic.ui.utils.UiHelper
import io.github.teamclouday.AndroidMic.utils.Either
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking


class MainViewModel : ViewModel() {

    private val TAG = "MainViewModel"


    val prefs: AppPreferences = AndroidMicApp.appModule.appPreferences
    val uiHelper: UiHelper = AndroidMicApp.appModule.uiHelper

    private var service: Messenger? = null
    private var isBound = false

    lateinit var handlerThread: HandlerThread
    private lateinit var messenger: Messenger
    lateinit var messengerLooper: Looper


    val textLog = mutableStateOf("")

    val isStreamStarted = mutableStateOf(false)
    val isButtonConnectClickable = mutableStateOf(false)

    val isMuted = mutableStateOf(false)

    init {
        Log.d(TAG, "init")
    }

    private inner class ReplyHandler(looper: Looper) : Handler(looper) {
        override fun handleMessage(msg: Message) {

            val data = ResponseData.fromMessage(msg)

            when (data.kind) {
                ResponseKind.Standard -> {
                    data.isConnected?.let {
                        isButtonConnectClickable.value = true
                        isStreamStarted.value = it
                    }

                    data.isMuted?.let {
                        isMuted.value = it
                    }

                    data.msg?.let {
                        addLogMessage(it)
                    }
                }
            }
        }
    }

    fun handlerServiceResponse() {
        handlerThread =
            HandlerThread("MainViewModelResponseHandler", Process.THREAD_PRIORITY_BACKGROUND)
        handlerThread.start()
        messengerLooper = handlerThread.looper
        messenger = Messenger(ReplyHandler(messengerLooper))
    }

    fun refreshAppVariables() {
        isBound = AndroidMicApp.isBound
        service = AndroidMicApp.service

        isStreamStarted.value = false
        isButtonConnectClickable.value = false
        isMuted.value = false

        val msg = CommandData(Command.BindCheck).toCommandMsg()
        msg.replyTo = messenger
        service?.send(msg)
    }

    fun onMuteSwitch() {
        if (!isBound) return

        val message = if (isMuted.value) {
            isMuted.value = false
            CommandData(Command.Unmute)
        } else {
            isMuted.value = true
            CommandData(Command.Mute)
        }.toCommandMsg()

        message.replyTo = messenger
        service?.send(message)
    }

    fun onConnectButton(): Dialogs? {
        if (!isBound) return null
        isMuted.value = false
        val message = if (isStreamStarted.value) {
            Log.d(TAG, "onConnectButton: stop stream")
            CommandData(Command.StopStream)
        } else {
            val res = runBlocking {
                CommandData.fromPref(prefs, Command.StartStream)
            }
            val data = when (res) {
                is Either.Left<CommandData> -> {
                    res.value
                }

                is Either.Right<Dialogs> -> {
                    uiHelper.makeToast(
                        uiHelper.getString(R.string.invalid_ip_port)
                    )
                    return res.value
                }
            }

            Log.d(TAG, "onConnectButton: start stream")

            data
        }.toCommandMsg()

        // lock button to avoid duplicate events
        isButtonConnectClickable.value = false

        message.replyTo = messenger
        service?.send(message)

        return null
    }

    fun setIpPort(ip: String, port: String) {
        viewModelScope.launch {
            prefs.ip.update(ip)
            prefs.port.update(port)
        }
    }

    fun setMode(mode: Mode) {
        viewModelScope.launch {
            prefs.mode.update(mode)
        }
    }

    fun setSampleRate(sampleRate: SampleRates) {
        viewModelScope.launch {
            prefs.sampleRate.update(sampleRate)
        }
    }

    fun setChannelCount(channelCount: ChannelCount) {
        viewModelScope.launch {
            prefs.channelCount.update(channelCount)
        }
    }

    fun setAudioFormat(audioFormat: AudioFormat) {
        viewModelScope.launch {
            prefs.audioFormat.update(audioFormat)
        }
    }


    fun setTheme(theme: Themes) {
        viewModelScope.launch {
            prefs.theme.update(theme)
        }
    }

    fun setDynamicColor(dynamicColor: Boolean) {
        viewModelScope.launch {
            prefs.dynamicColor.update(dynamicColor)
        }
    }

    fun cleanLog() {
        textLog.value = ""
    }


    // ask foreground service for current status
    fun askForStatus() {
        if (!isBound) return
        val message = Message.obtain()
        message.what = Command.GetStatus.ordinal
        message.replyTo = messenger
        service?.send(message)
    }


    // helper function to append log message to textview
    private fun addLogMessage(message: String) {
        textLog.value = textLog.value + message + "\n"
    }

    var perms: List<String> = listOf()
    fun showPermissionDialog(perms: List<String>) {
        this.perms = perms
    }
}