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
import io.github.teamclouday.AndroidMic.domain.service.Response
import io.github.teamclouday.AndroidMic.domain.service.ResponseData
import io.github.teamclouday.AndroidMic.domain.service.ServiceState
import io.github.teamclouday.AndroidMic.ui.utils.UiHelper
import io.github.teamclouday.AndroidMic.utils.checkIp
import io.github.teamclouday.AndroidMic.utils.checkPort
import kotlinx.coroutines.launch


class MainViewModel : ViewModel() {

    private val TAG = "MainViewModel"


    val prefs: AppPreferences = AndroidMicApp.appModule.appPreferences
    val uiHelper: UiHelper = AndroidMicApp.appModule.uiHelper

    private var mService: Messenger? = null
    private var mBound = false

    lateinit var handlerThread: HandlerThread
    private lateinit var mMessenger: Messenger
    lateinit var mMessengerLooper: Looper
    private lateinit var mMessengerHandler: ReplyHandler


    val textLog = mutableStateOf("")

    val isStreamStarted = mutableStateOf(false)
    val isButtonConnectClickable = mutableStateOf(false)

    init {
        Log.d(TAG, "init")
    }

    private inner class ReplyHandler(looper: Looper) : Handler(looper) {
        override fun handleMessage(msg: Message) {

            val data = ResponseData.fromMessage(msg);

            when (Response.entries[msg.what]) {
                Response.Standard -> {
                    data.state?.let {
                        isButtonConnectClickable.value = true
                        isStreamStarted.value = it == ServiceState.Connected
                    }

                    data.msg?.let {
                        addLogMessage(it)
                    }
                }
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

    fun refreshAppVariables() {
        mBound = AndroidMicApp.mBound
        mService = AndroidMicApp.mService

        isStreamStarted.value = false
        isButtonConnectClickable.value = false

        val msg = CommandData(Command.BindCheck).toCommandMsg()
        msg.replyTo = mMessenger
        mService?.send(msg)
    }

    fun onConnectButton(): Dialogs? {
        if (!mBound) return null
        val reply = if (isStreamStarted.value) {
            Log.d(TAG, "onConnectButton: stop stream")
            CommandData(Command.StopStream).toCommandMsg()
        } else {
            val ip = prefs.ip.getBlocking()
            val port = prefs.port.getBlocking()
            val mode = prefs.mode.getBlocking()

            val data = CommandData(
                command = Command.StartStream,
                sampleRate = prefs.sampleRate.getBlocking(),
                channelCount = prefs.channelCount.getBlocking(),
                audioFormat = prefs.audioFormat.getBlocking(),
                mode = mode
            )

            when (mode) {
                Mode.WIFI, Mode.UDP -> {
                    if (!checkIp(ip) || !checkPort(port)) {
                        uiHelper.makeToast(
                            uiHelper.getString(R.string.invalid_ip_port)
                        )
                        return Dialogs.IpPort
                    }
                    data.ip = ip
                    data.port = port.toInt()
                }

                else -> {}
            }

            Log.d(TAG, "onConnectButton: start stream")
            // lock button to avoid duplicate events
            isButtonConnectClickable.value = false

            data.toCommandMsg()
        }

        reply.replyTo = mMessenger
        mService?.send(reply)

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
        if (!mBound) return
        val reply = Message.obtain()
        reply.what = Command.GetStatus.ordinal
        reply.replyTo = mMessenger
        mService?.send(reply)
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