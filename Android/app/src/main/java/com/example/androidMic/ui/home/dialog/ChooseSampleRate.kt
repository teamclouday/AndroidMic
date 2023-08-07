package com.example.androidMic.ui.home.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.example.androidMic.ui.Dialogs
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.SampleRates
import com.example.androidMic.ui.States
import com.example.androidMic.ui.components.ManagerButton

@Composable
fun DialogSampleRate(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    if (uiStates.dialogVisible == Dialogs.SampleRates) {
        Dialog(
            onDismissRequest = { mainViewModel.showDialog(Dialogs.None) }
        ) {
            Surface(
                modifier = Modifier.fillMaxWidth(0.9f),
                shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface,
                contentColor = MaterialTheme.colorScheme.onSurface
            ) {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {

                    Spacer(modifier = Modifier.height(25.dp))


                    ManagerButton(
                        onClick = { mainViewModel.setSampleRate(SampleRates.S16000) },
                        text = "16000",
                        modifier = Modifier.fillMaxWidth(0.8f)
                    )

                    Spacer(modifier = Modifier.height(20.dp))


                    ManagerButton(
                        onClick = { mainViewModel.setSampleRate(SampleRates.S48000) },
                        text = "48000",
                        modifier = Modifier.fillMaxWidth(0.8f)
                    )

                    Spacer(modifier = Modifier.height(25.dp))
                }
            }
        }
    }
}