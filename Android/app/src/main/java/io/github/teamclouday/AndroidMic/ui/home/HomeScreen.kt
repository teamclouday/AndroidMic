package io.github.teamclouday.AndroidMic.ui.home

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalDrawerSheet
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.Scaffold
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
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.rememberMultiplePermissionsState
import io.github.teamclouday.AndroidMic.Dialogs
import io.github.teamclouday.AndroidMic.Mode
import io.github.teamclouday.AndroidMic.R
import io.github.teamclouday.AndroidMic.ui.MainViewModel
import io.github.teamclouday.AndroidMic.ui.components.ManagerButton
import io.github.teamclouday.AndroidMic.ui.home.dialog.DialogIpPort
import io.github.teamclouday.AndroidMic.ui.utils.WindowInfo
import io.github.teamclouday.AndroidMic.ui.utils.getBluetoothPermission
import io.github.teamclouday.AndroidMic.ui.utils.getRecordAudioPermission
import io.github.teamclouday.AndroidMic.ui.utils.getUsbPermission
import io.github.teamclouday.AndroidMic.ui.utils.getWifiPermission
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
            ModalDrawerSheet {
                DrawerBody(vm)
            }
        }
    ) {
        Scaffold(
            contentWindowInsets = WindowInsets.systemBars,
            topBar = {
                if (currentWindowInfo.screenHeightInfo != WindowInfo.WindowType.Compact)
                    AppBar(
                        onNavigationIconClick = {
                            scope.launch { drawerState.open() }
                        },
                    )
            }
        ) { padding ->

            if (currentWindowInfo.screenWidthInfo == WindowInfo.WindowType.Compact) {

                Column(
                    modifier = Modifier
                        .padding(padding)
                        .fillMaxSize(),
                    verticalArrangement = Arrangement.SpaceBetween,
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Log(
                        vm = vm,
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxWidth()
                            .padding(horizontal = 15.dp)
                            .padding(top = 15.dp)
                    )
                    ConnectButton(
                        vm = vm,
                        modifier = Modifier
                    )
                }

            } else {
                Row(
                    modifier = Modifier
                        .padding(padding)
                        .fillMaxSize(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically

                ) {
                    Log(
                        vm = vm,
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxHeight()
                            .padding(vertical = 15.dp)
                            .padding(start = 15.dp)
                    )

                    ConnectButton(
                        vm = vm,
                        modifier = Modifier
                    )
                }
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

@OptIn(ExperimentalPermissionsApi::class)
@Composable
private fun ConnectButton(
    vm: MainViewModel,
    modifier: Modifier
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

    Column(
        modifier = modifier
            .padding(vertical = 15.dp, horizontal = 15.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
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
}

