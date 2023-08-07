package com.example.androidMic.domain.service

class Command {
    companion object {
        const val COMMAND_START_STREAM: Int = 1
        const val COMMAND_STOP_STREAM: Int = 2
        const val COMMAND_START_AUDIO: Int = 3
        const val COMMAND_STOP_AUDIO: Int = 4

        const val COMMAND_GET_STATUS: Int = 30
        const val COMMAND_DISC_STREAM: Int = 40
    }
}