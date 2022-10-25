package com.example.androidMic.ui.home

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.DrawerValue
import androidx.compose.material.ModalDrawer
import androidx.compose.material.rememberDrawerState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.example.androidMic.R
import com.example.androidMic.ui.Event
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.components.ManagerButton
import com.example.androidMic.ui.utils.WindowInfo
import com.example.androidMic.ui.utils.getBluetoothPermission
import com.example.androidMic.ui.utils.getRecordAudioPermission
import com.example.androidMic.ui.utils.getWifiPermission
import com.example.androidMic.utils.Modes.Companion.MODE_BLUETOOTH
import com.example.androidMic.utils.Modes.Companion.MODE_USB
import com.example.androidMic.utils.Modes.Companion.MODE_WIFI
import com.example.androidMic.utils.States
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.rememberMultiplePermissionsState
import kotlinx.coroutines.launch

@Composable
fun HomeScreen(mainViewModel: MainViewModel, currentWindowInfo: WindowInfo) {

    val uiStates = mainViewModel.uiStates.collectAsState()

    val drawerState = rememberDrawerState(DrawerValue.Closed)
    val scope = rememberCoroutineScope()

    BackHandler(enabled = drawerState.isOpen) {
        scope.launch { drawerState.close() }
    }

    //TODO("change ModalDrawer to material3 when api will be better")
    ModalDrawer(
        drawerState = drawerState,
        gesturesEnabled = true,

        drawerContent = {
            Box(
                Modifier
                    .fillMaxSize()
                    .background(color = MaterialTheme.colorScheme.background)
            ) {
                Column {
                    DrawerBody(mainViewModel, uiStates.value)
                }
            }
        },
        content = {
            Box(
                modifier = Modifier
                    .background(color = MaterialTheme.colorScheme.background)
                    .fillMaxSize()
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                ) {
                    if (currentWindowInfo.screenWidthInfo == WindowInfo.WindowType.Compact) {
                        AppBar(onNavigationIconClick = {
                            scope.launch { drawerState.open() }
                        })
                        Log(mainViewModel, uiStates.value, currentWindowInfo)
                        Column(
                            Modifier.fillMaxSize(),
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.SpaceAround
                        ) {
                            ButtonConnect(
                                mainViewModel = mainViewModel,
                                uiStates = uiStates.value
                            )
                            SwitchAudio(mainViewModel = mainViewModel, uiStates = uiStates.value)
                        }
                    } else {
                        Row {
                            Log(mainViewModel, uiStates.value, currentWindowInfo)
                            Column(
                                modifier = Modifier
                                    .fillMaxSize(),
                                horizontalAlignment = Alignment.CenterHorizontally,
                                verticalArrangement = Arrangement.Center
                            ) {
                                ButtonConnect(
                                    mainViewModel = mainViewModel,
                                    uiStates = uiStates.value
                                )
                                Spacer(modifier = Modifier.height(10.dp))
                                SwitchAudio(
                                    mainViewModel = mainViewModel,
                                    uiStates = uiStates.value
                                )
                            }
                        }
                    }
                }
            }
        }
    )
}


@Composable
private fun Log(
    mainViewModel: MainViewModel,
    uiStates: States.UiStates,
    currentWindowInfo: WindowInfo
) {

    val modifier: Modifier =
        // for split screen
        if (currentWindowInfo.screenHeightInfo == WindowInfo.WindowType.Compact &&
            currentWindowInfo.screenWidthInfo == WindowInfo.WindowType.Compact
        ) {
            Modifier
                .fillMaxWidth()
                .fillMaxHeight(0.60f)
                .padding(16.dp)
        } else {
            // portrait mode
            if (currentWindowInfo.screenWidthInfo == WindowInfo.WindowType.Compact) {
                Modifier
                    .fillMaxWidth()
                    .fillMaxHeight(0.82f)
                    .padding(16.dp)
            }
            // landscape mode
            else {
                Modifier
                    .fillMaxWidth(0.70f)
                    .fillMaxHeight()
                    .padding(start = 16.dp, top = 16.dp, bottom = 16.dp)
            }
        }


    Box(
        modifier = modifier
            .background(color = MaterialTheme.colorScheme.secondary)
            .pointerInput(Unit) {
                detectTapGestures(
                    onDoubleTap = {
                        mainViewModel.onEvent(Event.CleanLog)
                    }
                )
            }
    )
    {
        Text(
            text = uiStates.textLog,
            color = MaterialTheme.colorScheme.onSecondary,
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier
                .verticalScroll(ScrollState(Int.MAX_VALUE))
                .padding(10.dp)
        )
    }
}


@OptIn(ExperimentalPermissionsApi::class)
@Composable
private fun ButtonConnect(
    mainViewModel: MainViewModel,
    uiStates: States.UiStates
) {
    val wifiPermissionsState = rememberMultiplePermissionsState(
        permissions = getWifiPermission()
    )
    val bluetoothPermissionsState = rememberMultiplePermissionsState(
        permissions = getBluetoothPermission()
    )

    ManagerButton(
        onClick = {
            when (uiStates.mode) {
                MODE_WIFI -> {
                    if (!wifiPermissionsState.allPermissionsGranted)
                        wifiPermissionsState.launchMultiplePermissionRequest()
                    else
                        mainViewModel.onEvent(Event.ConnectButton)
                }
                MODE_BLUETOOTH -> {
                    if (!bluetoothPermissionsState.allPermissionsGranted)
                        bluetoothPermissionsState.launchMultiplePermissionRequest()
                    else
                        mainViewModel.onEvent(Event.ConnectButton)
                }
                MODE_USB -> {

                }
            }
        },
        text =
        if (uiStates.isStreamStarted)
            stringResource(id = R.string.disconnect)
        else
            stringResource(id = R.string.connect),
        enabled = uiStates.buttonConnectIsClickable
    )
}

@OptIn(ExperimentalPermissionsApi::class)
@Composable
private fun SwitchAudio(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    val permissionsState = rememberMultiplePermissionsState(
        permissions = getRecordAudioPermission()
    )

    Row(
        modifier = Modifier
            .fillMaxWidth(),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = stringResource(id = R.string.turn_audio),
            color = MaterialTheme.colorScheme.onBackground,
            style = MaterialTheme.typography.labelLarge
        )

        Spacer(Modifier.width(12.dp))

        Switch(
            checked = uiStates.isAudioStarted,
            onCheckedChange = {
                // check for audio permission
                if (!permissionsState.allPermissionsGranted)
                    permissionsState.launchMultiplePermissionRequest()
                else
                    mainViewModel.onEvent(Event.AudioSwitch)

            },
            modifier = Modifier,
            enabled = uiStates.switchAudioIsClickable
        )
    }
}