package com.example.androidMic.ui.home

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.rememberDrawerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import com.example.androidMic.R
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.Modes
import com.example.androidMic.ui.States
import com.example.androidMic.ui.components.ManagerButton
import com.example.androidMic.ui.utils.WindowInfo
import com.example.androidMic.ui.utils.getBluetoothPermission
import com.example.androidMic.ui.utils.getRecordAudioPermission
import com.example.androidMic.ui.utils.getUsbPermission
import com.example.androidMic.ui.utils.getWifiPermission
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

    ModalNavigationDrawer(
        drawerState = drawerState,
        gesturesEnabled = true,

        drawerContent = {
            DrawerBody(mainViewModel, uiStates.value)
        }
    ) {
        ConstraintLayout(
            modifier = Modifier
                .fillMaxSize()
                .background(color = MaterialTheme.colorScheme.background)
        ) {
            val (appBar, interactionButton, log) = createRefs()

            if (currentWindowInfo.screenWidthInfo == WindowInfo.WindowType.Compact) {
                AppBar(
                    onNavigationIconClick = {
                        scope.launch { drawerState.open() }
                    },
                    modifier = Modifier
                        .constrainAs(appBar) {
                            top.linkTo(parent.top)
                            width = Dimension.matchParent
                        }
                )

                Log(
                    mainViewModel = mainViewModel,
                    uiStates = uiStates.value,
                    modifier = Modifier
                        .constrainAs(log) {
                            linkTo(top = appBar.bottom, bottom = interactionButton.top)
                            width = Dimension.matchParent
                            height = Dimension.fillToConstraints
                        }
                        .padding(horizontal = 15.dp)
                        .padding(top = 15.dp)
                )
                InteractionButton(
                    mainViewModel = mainViewModel,
                    uiStates = uiStates.value,
                    modifier = Modifier
                        .constrainAs(interactionButton) {
                            bottom.linkTo(parent.bottom)
                            width = Dimension.matchParent
                        }
                )

            } else {
                var appBarEnabled = false
                if (currentWindowInfo.screenHeightInfo != WindowInfo.WindowType.Compact) {
                    appBarEnabled = true
                    AppBar(
                        onNavigationIconClick = {
                            scope.launch { drawerState.open() }
                        },
                        modifier = Modifier
                            .constrainAs(appBar) {
                                top.linkTo(parent.top)
                                width = Dimension.matchParent
                            }
                    )
                }

                Log(
                    mainViewModel = mainViewModel,
                    uiStates = uiStates.value,
                    modifier = Modifier
                        .constrainAs(log) {
                            linkTo(start = parent.start, end = interactionButton.start)
                            linkTo(
                                top = if (appBarEnabled) appBar.bottom else parent.top,
                                bottom = parent.bottom
                            )
                            width = Dimension.fillToConstraints
                            height = Dimension.fillToConstraints
                        }
                        .padding(vertical = 15.dp)
                        .padding(start = 15.dp)
                )

                InteractionButton(
                    mainViewModel = mainViewModel,
                    uiStates = uiStates.value,
                    modifier = Modifier
                        .constrainAs(interactionButton) {
                            end.linkTo(parent.end)
                            linkTo(
                                top = if (appBarEnabled) appBar.bottom else parent.top,
                                bottom = parent.bottom
                            )
                        }
                )
            }
        }
    }
}


@Composable
private fun Log(
    mainViewModel: MainViewModel,
    uiStates: States.UiStates,
    modifier: Modifier
) {
    Box(
        modifier = modifier
            .background(color = MaterialTheme.colorScheme.secondary)
            .pointerInput(Unit) {
                detectTapGestures(
                    onDoubleTap = {
                        mainViewModel.cleanLog()
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
                .padding(15.dp)
        )
    }
}

@Composable
private fun InteractionButton(
    mainViewModel: MainViewModel,
    uiStates: States.UiStates,
    modifier: Modifier
) {
    Column(
        modifier = modifier
            .padding(vertical = 15.dp, horizontal = 15.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        ButtonConnect(
            mainViewModel = mainViewModel,
            uiStates = uiStates
        )
        Spacer(modifier = Modifier.height(10.dp))
        SwitchAudio(mainViewModel = mainViewModel, uiStates = uiStates)
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
    val usbPermissionsState = rememberMultiplePermissionsState(
        permissions = getUsbPermission()
    )

    ManagerButton(
        onClick = {
            when (uiStates.mode) {
                Modes.WIFI -> {
                    if (!wifiPermissionsState.allPermissionsGranted)
                        wifiPermissionsState.launchMultiplePermissionRequest()
                    else
                        mainViewModel.onConnectButton()
                }

                Modes.BLUETOOTH -> {
                    if (!bluetoothPermissionsState.allPermissionsGranted)
                        bluetoothPermissionsState.launchMultiplePermissionRequest()
                    else
                        mainViewModel.onConnectButton()
                }

                Modes.USB -> {
                    if (!usbPermissionsState.allPermissionsGranted)
                        usbPermissionsState.launchMultiplePermissionRequest()
                    else
                        mainViewModel.onConnectButton()
                }

                Modes.UDP -> {
                    mainViewModel.onConnectButton()
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
                    mainViewModel.onAudioSwitch()

            },
            enabled = uiStates.switchAudioIsClickable
        )
    }
}