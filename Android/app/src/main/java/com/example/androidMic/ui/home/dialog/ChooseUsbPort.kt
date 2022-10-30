package com.example.androidMic.ui.home.dialog

import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.example.androidMic.R
import com.example.androidMic.ui.Event
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.components.ManagerButton
import com.example.androidMic.ui.components.ManagerOutlinedTextField
import com.example.androidMic.ui.utils.Preferences.Companion.DEFAULT_USB_PORT
import com.example.androidMic.utils.States

@Composable
fun DialogUsbPort(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    val tempPort = remember {
        mutableStateOf(uiStates.usbPort)
    }

    if (uiStates.dialogUsbPortIsVisible) {
        Dialog(
            onDismissRequest = {
                tempPort.value = uiStates.usbPort
                mainViewModel.onEvent(Event.DismissDialog(R.string.drawerUsbPort))
            }
        ) {
            Surface(
                shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface,
                contentColor = MaterialTheme.colorScheme.onSurface
            ) {
                Column(
                    horizontalAlignment = Alignment.End
                ) {
                    Spacer(modifier = Modifier.height(25.dp))
                    // reset button
                    ManagerButton(
                        onClick = {
                            tempPort.value = DEFAULT_USB_PORT.toString()
                        },
                        text = stringResource(id = R.string.reset),
                        modifier = Modifier.padding(end = 10.dp),
                    )
                    Spacer(modifier = Modifier.height(20.dp))
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        // port field
                        ManagerOutlinedTextField(tempPort, R.string.dialog_port)

                        Spacer(modifier = Modifier.height(20.dp))

                        // save Button
                        ManagerButton(
                            onClick = {
                                mainViewModel.onEvent(
                                    Event.SetUsbPort(
                                        tempPort.value
                                    )
                                )
                            },
                            text = stringResource(id = R.string.save),
                            modifier = Modifier.fillMaxWidth(0.6f)
                        )

                        Spacer(modifier = Modifier.height(10.dp))

                        // cancel Button
                        ManagerButton(
                            onClick = {
                                tempPort.value = uiStates.usbPort
                                mainViewModel.onEvent(Event.DismissDialog(R.string.drawerUsbPort))
                            },
                            text = stringResource(id = R.string.cancel),
                            modifier = Modifier.fillMaxWidth(0.6f)
                        )
                        Spacer(modifier = Modifier.height(25.dp))
                    }
                }
            }
        }
    }
}
