package com.example.androidmic.ui

sealed class Event {
    object ConnectButton: Event()
    object AudioSwitch: Event()

    data class ShowDialog(val id: Int): Event()
    data class DismissDialog(val id: Int): Event()

    data class SetMode(val mode: Int): Event()
    data class SetIpPort(val ip: String, val port: String): Event()
}