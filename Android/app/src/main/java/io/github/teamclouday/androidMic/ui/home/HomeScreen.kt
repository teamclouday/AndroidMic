package io.github.teamclouday.androidMic.ui.home

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.Text
import androidx.compose.material3.rememberDrawerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import io.github.teamclouday.androidMic.Dialogs
import io.github.teamclouday.androidMic.Mode
import io.github.teamclouday.androidMic.R
import io.github.teamclouday.androidMic.ui.MainViewModel
import io.github.teamclouday.androidMic.ui.components.ManagerButton
import io.github.teamclouday.androidMic.ui.home.dialog.DialogIpPort
import io.github.teamclouday.androidMic.ui.utils.WindowInfo
import io.github.teamclouday.androidMic.ui.utils.getBluetoothPermission
import io.github.teamclouday.androidMic.ui.utils.getRecordAudioPermission
import io.github.teamclouday.androidMic.ui.utils.getUsbPermission
import io.github.teamclouday.androidMic.ui.utils.getWifiPermission
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.rememberMultiplePermissionsState
import kotlinx.coroutines.launch

@Composable
fun HomeScreen(vm: MainViewModel, currentWindowInfo: WindowInfo) {

    val drawerState = rememberDrawerState(DrawerValue.Closed)
    val scope = rememberCoroutineScope()

    BackHandler(enabled = drawerState.isOpen) {
        scope.launch { drawerState.close() }
    }

    ModalNavigationDrawer(
        drawerState = drawerState,
        gesturesEnabled = true,

        drawerContent = {
            DrawerBody(vm)
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
                    vm = vm,
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
                    mainViewModel = vm,
                    modifier = Modifier
                        .constrainAs(interactionButton) {
                            bottom.linkTo(parent.bottom)
                            width = Dimension.matchParent
                            height = Dimension.percent(0.15f)
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
                    vm = vm,
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
                    mainViewModel = vm,
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
    vm: MainViewModel,
    modifier: Modifier
) {
    Box(
        modifier = modifier
            .background(color = MaterialTheme.colorScheme.secondary)
            .pointerInput(Unit) {
                detectTapGestures(
                    onDoubleTap = {
                        vm.cleanLog()
                    }
                )
            }
    )
    {
        Text(
            text = vm.textLog.value,
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
    modifier: Modifier
) {
    Column(
        modifier = modifier
            .padding(vertical = 15.dp, horizontal = 15.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        ButtonConnect(
            vm = mainViewModel,
        )
    }
}

@OptIn(ExperimentalPermissionsApi::class)
@Composable
private fun ButtonConnect(
    vm: MainViewModel,
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

    val audioPermissionsState = rememberMultiplePermissionsState(
        permissions = getRecordAudioPermission()
    )


    val dialogIpPortExpanded = rememberSaveable {
        mutableStateOf(false)
    }
    DialogIpPort(vm = vm, expanded = dialogIpPortExpanded)

    ManagerButton(
        onClick = {
            when (vm.prefs.mode.getBlocking()) {
                Mode.WIFI -> {
                    if (!wifiPermissionsState.allPermissionsGranted)
                        return@ManagerButton wifiPermissionsState.launchMultiplePermissionRequest()
                }

//                Modes.BLUETOOTH -> {
//                    if (!bluetoothPermissionsState.allPermissionsGranted)
//                        return@ManagerButton bluetoothPermissionsState.launchMultiplePermissionRequest()
//                }

                Mode.USB -> {
                    if (!usbPermissionsState.allPermissionsGranted)
                        return@ManagerButton usbPermissionsState.launchMultiplePermissionRequest()
                }

                else -> {}
            }


            if (!audioPermissionsState.allPermissionsGranted)
                return@ManagerButton audioPermissionsState.launchMultiplePermissionRequest()

            val dialog = vm.onConnectButton()

            if (dialog != null) {
                when (dialog) {
                    Dialogs.IpPort -> {
                        dialogIpPortExpanded.value = true
                    }
                }
            }
        },
        text =
        if (vm.isStreamStarted.value)
            stringResource(id = R.string.disconnect)
        else
            stringResource(id = R.string.connect),
        enabled = vm.isButtonConnectClickable.value
    )
}

