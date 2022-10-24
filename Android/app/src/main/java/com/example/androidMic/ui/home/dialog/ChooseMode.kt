package com.example.androidMic.ui.home.dialog

import androidx.compose.foundation.layout.*
import androidx.compose.material3.Divider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.example.androidMic.ui.Event
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.components.ManagerButton
import com.example.androidMic.utils.Modes
import com.example.androidMic.utils.States
import com.example.androidMic.R

@Composable
fun DialogMode(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    if (uiStates.dialogModesIsVisible) {
        Dialog(
            onDismissRequest = { mainViewModel.onEvent(Event.DismissDialog(R.string.drawerMode)) }
        ) {
            Surface(
                shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface,
                contentColor = MaterialTheme.colorScheme.onSurface
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {

                    Spacer(modifier = Modifier.height(10.dp))

                    // wifi
                    ManagerButton(
                        onClick = { mainViewModel.onEvent(Event.SetMode(Modes.MODE_WIFI)) },
                        text = stringResource(id = R.string.mode_wifi),
                        modifier = Modifier.fillMaxWidth(0.6F)
                    )

                    Divider(
                        modifier = Modifier.padding(10.dp),
                        color = MaterialTheme.colorScheme.onSurface
                    )

                    // bluetooth
                    ManagerButton(
                        onClick = { mainViewModel.onEvent(Event.SetMode(Modes.MODE_BLUETOOTH)) },
                        text = stringResource(id = R.string.mode_bluetooth),
                        modifier = Modifier.fillMaxWidth(0.6F)
                    )

                    Divider(
                        modifier = Modifier.padding(10.dp),
                        color = MaterialTheme.colorScheme.onSurface
                    )

                    // usb
                    ManagerButton(
                        onClick = { mainViewModel.onEvent(Event.SetMode(Modes.MODE_USB)) },
                        text = stringResource(id = R.string.mode_usb),
                        modifier = Modifier.fillMaxWidth(0.6F)
                    )
                    Spacer(modifier = Modifier.height(10.dp))
                }
            }
        }
    }
}