package com.example.androidmic.utils



data class CommandService(
    val dic: Map<Int, String> = mapOf(
        1  to "COMMAND_START_STREAM",
        2  to "COMMAND_STOP_STREAM",
        3  to "COMMAND_START_AUDIO",
        4  to "COMMAND_STOP_AUDIO",

        10 to "COMMAND_SET_IP_PORT",
        20 to "COMMAND_SET_MODE",
        30 to "COMMAND_GET_STATUS",
        40 to "COMMAND_DISC_STREAM")
)

data class DebugModes(
    val dic: Map<Int, String> = mapOf(
        1  to "MODE_WIFI",
        2  to "MODE_BLUETOOTH",
        3  to "MODE_USB")
)