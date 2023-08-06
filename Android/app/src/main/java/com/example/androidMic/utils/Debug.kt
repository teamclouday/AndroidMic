package com.example.androidMic.utils

import android.util.Log
import com.example.androidMic.ui.utils.WindowInfo

// debug file which help for Log
// todo: remove this file

data class CommandService(
    val dic: Map<Int, String> = mapOf(
        1 to "COMMAND_START_STREAM",
        2 to "COMMAND_STOP_STREAM",
        3 to "COMMAND_START_AUDIO",
        4 to "COMMAND_STOP_AUDIO",

        10 to "COMMAND_SET_IP_PORT",
        20 to "COMMAND_SET_MODE",
        30 to "COMMAND_GET_STATUS",
        40 to "COMMAND_DISC_STREAM"
    )
)

data class DebugModes(
    val dic: Map<Int, String> = mapOf(
        1 to "WIFI",
        2 to "BLUETOOTH",
        3 to "USB",
        4 to "UDP"
    )
)

fun showWindowsInfo(currentWindowInfo: WindowInfo) {

    var log = "Width: "
    log += when (currentWindowInfo.screenWidthInfo) {
        WindowInfo.WindowType.Compact -> "Compact"
        WindowInfo.WindowType.Medium -> "Medium"
        WindowInfo.WindowType.Expanded -> "Expanded"
    }

    log += ", Height: "
    log += when (currentWindowInfo.screenHeightInfo) {
        WindowInfo.WindowType.Compact -> "Compact"
        WindowInfo.WindowType.Medium -> "Medium"
        WindowInfo.WindowType.Expanded -> "Expanded"
    }
    Log.d("WindowsInfo", log)
}