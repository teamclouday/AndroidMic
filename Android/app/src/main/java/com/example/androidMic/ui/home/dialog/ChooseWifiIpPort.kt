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
import com.example.androidMic.ui.utils.Preferences.Companion.DEFAULT_IP
import com.example.androidMic.ui.utils.Preferences.Companion.DEFAULT_PORT
import com.example.androidMic.ui.States

@Composable
fun DialogWifiIpPort(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    val tempIP = remember {
        mutableStateOf(uiStates.IP)
    }
    val tempPort = remember {
        mutableStateOf(uiStates.PORT)
    }

    if (uiStates.dialogIpPortIsVisible) {
        Dialog(
            onDismissRequest = {
                tempIP.value = uiStates.IP; tempPort.value = uiStates.PORT
                mainViewModel.onEvent(Event.DismissDialog(R.string.drawerWifiIpPort))
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
                            tempIP.value = DEFAULT_IP; tempPort.value = DEFAULT_PORT.toString()
                        },
                        text = stringResource(id = R.string.reset),
                        modifier = Modifier.padding(end = 10.dp),
                    )
                    Spacer(modifier = Modifier.height(20.dp))
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        // ip field
                        ManagerOutlinedTextField(tempIP, R.string.dialog_ip)

                        Spacer(modifier = Modifier.height(10.dp))

                        // port field
                        ManagerOutlinedTextField(tempPort, R.string.dialog_port)

                        Spacer(modifier = Modifier.height(20.dp))

                        // save Button
                        ManagerButton(
                            onClick = {
                                mainViewModel.onEvent(
                                    Event.SetWifiIpPort(
                                        tempIP.value,
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
                                tempIP.value = uiStates.IP; tempPort.value = uiStates.PORT
                                mainViewModel.onEvent(Event.DismissDialog(R.string.drawerWifiIpPort))
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
