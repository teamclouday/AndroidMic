package com.example.androidMic.ui.home.dialog

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.example.androidMic.R
import com.example.androidMic.ui.Event
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.components.ManagerButton
import com.example.androidMic.ui.utils.Preferences.Companion.DEFAULT_IP
import com.example.androidMic.ui.utils.Preferences.Companion.DEFAULT_PORT
import com.example.androidMic.ui.utils.Preferences.Companion.DEFAULT_USB_PORT
import com.example.androidMic.utils.States

@OptIn(ExperimentalMaterial3Api::class)
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
                    // reset button
                    ManagerButton(
                        onClick = {
                            tempPort.value = DEFAULT_USB_PORT.toString()
                        },
                        text = stringResource(id = R.string.reset),
                        modifier = Modifier.padding(10.dp),
                    )
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        // port field
                        OutlinedTextField(
                            modifier = Modifier.padding(10.dp),
                            value = tempPort.value,
                            onValueChange = { tempPort.value = it },
                            enabled = true,
                            singleLine = true,
                            label = { Text(stringResource(id = R.string.dialog_port)) },
                            textStyle = MaterialTheme.typography.bodyMedium,
                            colors = TextFieldDefaults.textFieldColors(
                                textColor = MaterialTheme.colorScheme.onSurface,
                                containerColor = MaterialTheme.colorScheme.surface
                            ),
                            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number)
                        )

                        Spacer(modifier = Modifier.height(15.dp))

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
                        Spacer(modifier = Modifier.height(10.dp))
                    }
                }
            }
        }
    }
}
