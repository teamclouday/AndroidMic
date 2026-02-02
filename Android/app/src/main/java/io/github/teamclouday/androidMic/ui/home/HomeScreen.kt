package io.github.teamclouday.androidMic.ui.home

import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.scrollable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalDrawerSheet
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Switch
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
import io.github.teamclouday.androidMic.Dialogs
import io.github.teamclouday.androidMic.Mode
import io.github.teamclouday.androidMic.R
import io.github.teamclouday.androidMic.ui.MainViewModel
import io.github.teamclouday.androidMic.ui.components.ManagerButton
import io.github.teamclouday.androidMic.ui.home.dialog.DialogIpPort
import io.github.teamclouday.androidMic.ui.utils.WindowInfo
import kotlinx.coroutines.launch


private const val TAG = "HomeScreen"

@Composable
fun HomeScreen(
    vm: MainViewModel,
    currentWindowInfo: WindowInfo,
    openAppSettings: () -> Unit
) {

    val drawerState = rememberDrawerState(DrawerValue.Closed)
    val scope = rememberCoroutineScope()

    BackHandler(enabled = drawerState.isOpen) {
        scope.launch { drawerState.close() }
    }

    ModalNavigationDrawer(
        drawerState = drawerState,
        gesturesEnabled = true,

        drawerContent = {
            ModalDrawerSheet(
                windowInsets = WindowInsets.safeDrawing,
                modifier = Modifier.scrollable(rememberScrollState(), Orientation.Vertical)
            ) {
                DrawerBody(vm)
            }
        }
    ) {
        Scaffold(
            contentWindowInsets = WindowInsets.safeDrawing,
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
                            .padding(all = 15.dp)
                    )

                    Spacer(modifier = Modifier.height(40.dp))
                    ConnectButton(
                        vm = vm,
                        modifier = Modifier,
                        openAppSettings = openAppSettings
                    )

                    if (vm.isStreamStarted.value) {
                        Spacer(modifier = Modifier.height(15.dp))

                        AudioSwitch(
                            vm = vm,
                        )
                    }

                    Spacer(modifier = Modifier.height(40.dp))
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
                            .padding(all = 15.dp)
                    )

                    Column(
                        modifier = Modifier
                            .padding(all = 15.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.Center
                    ) {
                        ConnectButton(
                            vm = vm,
                            modifier = Modifier,
                            openAppSettings = openAppSettings
                        )


                        if (vm.isStreamStarted.value) {
                            Spacer(modifier = Modifier.height(15.dp))
                            AudioSwitch(
                                vm = vm,
                            )
                        }
                    }
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

@Composable
private fun AudioSwitch(
    vm: MainViewModel,
) {

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
            checked = vm.isMuted.value,
            onCheckedChange = {
                vm.onMuteSwitch()
            }
        )
    }
}

@Composable
private fun ConnectButton(
    vm: MainViewModel,
    modifier: Modifier,
    openAppSettings: () -> Unit
) {

    val dialogIpPortExpanded = rememberSaveable {
        mutableStateOf(false)
    }
    val dialogPermissionRationalExpanded = rememberSaveable {
        mutableStateOf(false)
    }

    val permissionResultLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { perms ->

        val permsDeclined = perms.filter { !it.value }.map { it.key }.toList()

        if (permsDeclined.isNotEmpty()) {
            dialogPermissionRationalExpanded.value = true
            vm.showPermissionDialog(permsDeclined)
            return@rememberLauncherForActivityResult
        } else {
            dialogPermissionRationalExpanded.value = false
        }

        val dialog = vm.onConnectButton()

        if (dialog != null) {
            when (dialog) {
                Dialogs.IpPort -> {
                    dialogIpPortExpanded.value = true
                }
            }
        }
    }

    DialogIpPort(vm = vm, expanded = dialogIpPortExpanded)



    PermissionDialog(
        vm = vm,
        expanded = dialogPermissionRationalExpanded,
        onRequestPermissionAgain = {
            permissionResultLauncher.launch(vm.perms.toTypedArray())
        },
        openAppSettings = openAppSettings
    )


    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {

        ManagerButton(
            onClick = {

                val perms = getRecordAudioPermission()

                when (vm.prefs.mode.getBlocking()) {
                    Mode.WIFI -> {
                        perms.addAll(getWifiPermission())
                        permissionResultLauncher.launch(perms.toTypedArray())
                    }

                    Mode.USB -> {
                        perms.addAll(getUsbPermission())
                        permissionResultLauncher.launch(perms.toTypedArray())
                    }

                    else -> {
                        permissionResultLauncher.launch(perms.toTypedArray())
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

