package com.example.androidmic.ui.home.dialog

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.example.androidmic.R
import com.example.androidmic.ui.Event
import com.example.androidmic.ui.MainViewModel
import com.example.androidmic.ui.components.ManagerButton
import com.example.androidmic.utils.States

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DialogIpPort(mainViewModel: MainViewModel, uiStates: States.UiStates) {

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
                mainViewModel.onEvent(Event.DismissDialog(R.string.drawerIpPort))
            }
        ) {
            Surface(
                shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface,
                contentColor = MaterialTheme.colorScheme.onSurface
            ) {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    // ip field
                    OutlinedTextField(
                        modifier = Modifier.padding(10.dp),
                        value = tempIP.value,
                        onValueChange = { tempIP.value = it },
                        enabled = true,
                        singleLine = true,
                        label = { Text(stringResource(id = R.string.dialog_ip)) },
                        textStyle = MaterialTheme.typography.bodyMedium,
                        colors = TextFieldDefaults.textFieldColors(
                            textColor = MaterialTheme.colorScheme.onSurface,
                            containerColor = MaterialTheme.colorScheme.surface
                        )
                    )

                    Spacer(modifier = Modifier.height(10.dp))

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
                        )
                    )

                    Spacer(modifier = Modifier.height(15.dp))

                    // save Button
                    ManagerButton(
                        onClick = {
                            mainViewModel.onEvent(
                                Event.SetIpPort(
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
                            mainViewModel.onEvent(Event.DismissDialog(R.string.drawerIpPort))
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
