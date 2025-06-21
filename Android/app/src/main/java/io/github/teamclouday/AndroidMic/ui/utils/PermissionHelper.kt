package io.github.teamclouday.AndroidMic.ui.utils

import android.Manifest
import android.os.Build

fun getWifiPermission(): MutableList<String> {
    val list = mutableListOf<String>()

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU)
        list.add(Manifest.permission.POST_NOTIFICATIONS)

    return list
}

fun getBluetoothPermission(): MutableList<String> {
    val list = mutableListOf<String>()

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU)
        list.add(Manifest.permission.POST_NOTIFICATIONS)

    list.add(Manifest.permission.BLUETOOTH)

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S)
        list.add(Manifest.permission.BLUETOOTH_CONNECT)

    return list
}

fun getUsbPermission(): MutableList<String> {
    val list = mutableListOf<String>()

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU)
        list.add(Manifest.permission.POST_NOTIFICATIONS)

    return list
}

fun getRecordAudioPermission(): MutableList<String> {
    val list = mutableListOf<String>()

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU)
        list.add(Manifest.permission.POST_NOTIFICATIONS)

    list.add(Manifest.permission.RECORD_AUDIO)

    return list
}
