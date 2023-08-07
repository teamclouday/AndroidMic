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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.example.androidMic.R
import com.example.androidMic.ui.Event
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.components.ManagerButton
import com.example.androidMic.ui.Modes
import com.example.androidMic.ui.States

@Composable
fun DialogMode(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    if (uiStates.dialogModesIsVisible) {
        Dialog(
            onDismissRequest = { mainViewModel.onEvent(Event.DismissDialog(R.string.drawerMode)) }
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

                    // wifi
                    ManagerButton(
                        onClick = { mainViewModel.onEvent(Event.SetMode(Modes.WIFI)) },
                        text = stringResource(id = R.string.mode_wifi),
                        modifier = Modifier.fillMaxWidth(0.8f)
                    )

                    Spacer(modifier = Modifier.height(20.dp))

                    // bluetooth
                    ManagerButton(
                        onClick = { mainViewModel.onEvent(Event.SetMode(Modes.BLUETOOTH)) },
                        text = stringResource(id = R.string.mode_bluetooth),
                        modifier = Modifier.fillMaxWidth(0.8f)
                    )

                    Spacer(modifier = Modifier.height(20.dp))

                    // usb
                    ManagerButton(
                        onClick = { mainViewModel.onEvent(Event.SetMode(Modes.USB)) },
                        text = stringResource(id = R.string.mode_usb),
                        modifier = Modifier.fillMaxWidth(0.8f)
                    )

                    Spacer(modifier = Modifier.height(20.dp))

                    // udp
                    ManagerButton(
                        onClick = { mainViewModel.onEvent(Event.SetMode(Modes.UDP)) },
                        text = stringResource(id = R.string.mode_udp),
                        modifier = Modifier.fillMaxWidth(0.8f)
                    )

                    Spacer(modifier = Modifier.height(25.dp))
                }
            }
        }
    }
}