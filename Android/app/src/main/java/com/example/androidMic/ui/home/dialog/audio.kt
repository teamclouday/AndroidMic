package com.example.androidMic.ui.home.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import com.example.androidMic.AudioFormat
import com.example.androidMic.ChannelCount
import com.example.androidMic.SampleRates
import com.example.androidMic.ui.MainViewModel

@Composable
fun DialogSampleRate(
    vm: MainViewModel,
    expanded: MutableState<Boolean>,
) {
    ManagerDialog(
        expanded
    ) {
        DialogList(
            enum = SampleRates.entries,
            onClick = { vm.setSampleRate(it) },
            text = { it.value.toString() }
        )
    }
}

@Composable
fun DialogChannelCount(
    vm: MainViewModel,
    expanded: MutableState<Boolean>,
) {
    ManagerDialog(
        expanded
    ) {
        DialogList(
            enum = ChannelCount.entries,
            onClick = { vm.setChannelCount(it) },
            text = { it.toString() }
        )
    }
}

@Composable
fun DialogAudioFormat(
    vm: MainViewModel,
    expanded: MutableState<Boolean>,
) {
    ManagerDialog(
        expanded
    ) {
        DialogList(
            enum = AudioFormat.entries,
            onClick = { vm.setAudioFormat(it) },
            text = { it.toString() }
        )
    }
}