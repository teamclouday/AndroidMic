package com.example.androidMic.ui

open class Event {
    object ConnectButton : Event()
    object AudioSwitch : Event()

    data class ShowDialog(val id: Int) : Event()
    data class DismissDialog(val id: Int) : Event()

    data class SetMode(val mode: Modes) : Event()
    data class SetWifiIpPort(val ip: String, val port: String) : Event()
    data class SetUsbPort(val port: String) : Event()


    data class SetTheme(val theme: Themes) : Event()
    data class SetDynamicColor(val dynamicColor: Boolean) : Event()

    object CleanLog : Event()
}