package com.example.androidMic.ui.home.dialog

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.example.androidMic.ui.Dialogs
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.SampleRates
import com.example.androidMic.ui.States
import com.example.androidMic.ui.components.ManagerButton

@Composable
fun DialogSampleRate(mainViewModel: MainViewModel, uiStates: States.UiStates) {
    ManagerDialog(
        mainViewModel,
        uiStates,
        Dialogs.SampleRates
    ) {
        ManagerButton(
            onClick = { mainViewModel.setSampleRate(SampleRates.S16000) },
            text = SampleRates.S16000.value.toString(),
            modifier = Modifier.fillMaxWidth(0.8f)
        )

        DialogSpacer()

        ManagerButton(
            onClick = { mainViewModel.setSampleRate(SampleRates.S48000) },
            text = SampleRates.S48000.value.toString(),
            modifier = Modifier.fillMaxWidth(0.8f)
        )
    }
}