package com.example.androidMic.ui.home.dialog

import androidx.compose.runtime.Composable
import com.example.androidMic.ui.AudioFormat
import com.example.androidMic.ui.ChannelCount
import com.example.androidMic.ui.Dialogs
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.SampleRates
import com.example.androidMic.ui.States

@Composable
fun DialogSampleRate(mainViewModel: MainViewModel, uiStates: States.UiStates) {
    ManagerDialog(
        mainViewModel,
        uiStates,
        Dialogs.SampleRates
    ) {
        DialogList(
            enum = SampleRates.values().toList(),
            onClick = { mainViewModel.setSampleRate(it) },
            text = { it.value.toString() }
        )
    }
}

@Composable
fun DialogChannelCount(mainViewModel: MainViewModel, uiStates: States.UiStates) {
    ManagerDialog(
        mainViewModel,
        uiStates,
        Dialogs.ChannelCount
    ) {
        DialogList(
            enum = ChannelCount.values().toList(),
            onClick = { mainViewModel.setChannelCount(it) },
            text = { it.toString() }
        )
    }
}

@Composable
fun DialogAudioFormat(mainViewModel: MainViewModel, uiStates: States.UiStates) {
    ManagerDialog(
        mainViewModel,
        uiStates,
        Dialogs.AudioFormat
    ) {
        DialogList(
            enum = AudioFormat.values().toList(),
            onClick = { mainViewModel.setAudioFormat(it) },
            text = { it.toString() }
        )
    }
}