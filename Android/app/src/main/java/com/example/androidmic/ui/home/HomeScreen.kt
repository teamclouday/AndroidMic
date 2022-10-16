package com.example.androidmic.ui.home

import android.Manifest
import android.os.Build
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.example.androidmic.R
import com.example.androidmic.ui.Event
import com.example.androidmic.ui.MainViewModel
import com.example.androidmic.ui.components.ManagerButton
import com.example.androidmic.ui.home.drawer.DrawerBody
import com.example.androidmic.ui.home.drawer.DrawerHeader
import com.example.androidmic.ui.utils.WindowInfo
import com.example.androidmic.utils.Modes.Companion.MODE_BLUETOOTH
import com.example.androidmic.utils.Modes.Companion.MODE_USB
import com.example.androidmic.utils.Modes.Companion.MODE_WIFI
import com.example.androidmic.utils.States
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
                    .background(color = MaterialTheme.colorScheme.background)) {
                Column {
                    DrawerHeader()
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
                Column (
                    modifier = Modifier
                        .fillMaxSize(),
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    if(currentWindowInfo.screenWidthInfo == WindowInfo.WindowType.Compact) {
                        AppBar(onNavigationIconClick = {
                            scope.launch { drawerState.open() }
                        }
                        )
                        Log(uiStates.value, currentWindowInfo)
                        ButtonConnect(mainViewModel = mainViewModel, uiStates = uiStates.value)
                        SwitchAudio(mainViewModel = mainViewModel, states = uiStates.value)
                    }
                    else {
                        Row {
                            Log(uiStates.value, currentWindowInfo)
                            Column (
                                modifier = Modifier
                                    .fillMaxSize(),
                                horizontalAlignment = Alignment.CenterHorizontally,
                                verticalArrangement = Arrangement.Center
                            ){
                                ButtonConnect(mainViewModel = mainViewModel, uiStates = uiStates.value)
                                SwitchAudio(mainViewModel = mainViewModel, states = uiStates.value)
                            }
                        }
                    }
                }
            }
        }
    )
}


@Composable
private fun Log(states: States.UiStates, currentWindowInfo: WindowInfo) {

    val modifier : Modifier = if(currentWindowInfo.screenWidthInfo == WindowInfo.WindowType.Compact) {
        Modifier
            .fillMaxWidth()
            .fillMaxHeight(0.82f)
    } else {
        Modifier
            .fillMaxWidth(0.75f)
            .fillMaxHeight()
    }

    Box(
        modifier = modifier
            .padding(16.dp)
            .background(color = MaterialTheme.colorScheme.secondary))
    {
        Text(text = states.textLog,
            color = MaterialTheme.colorScheme.onSecondary,
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier
                .verticalScroll(ScrollState(0))
                .padding(10.dp)
        )
    }
}


@OptIn(ExperimentalPermissionsApi::class)
@Composable
private fun ButtonConnect(mainViewModel: MainViewModel, uiStates: States.UiStates) {
    val list = mutableListOf(
        Manifest.permission.BLUETOOTH
    )
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S)
        list.add(Manifest.permission.BLUETOOTH_CONNECT)
    val permissionsState = rememberMultiplePermissionsState(permissions = list)

    ManagerButton(
        onClick = {
            when(uiStates.mode) {
                MODE_BLUETOOTH -> {
                    if(!permissionsState.allPermissionsGranted)
                        permissionsState.launchMultiplePermissionRequest()
                    else
                        mainViewModel.onEvent(Event.ConnectButton)
                }
                MODE_WIFI -> {
                    mainViewModel.onEvent(Event.ConnectButton)
                }
                MODE_USB -> {

                }
            }
        },
        text =
        if(uiStates.isStreamStarted)
            stringResource(id = R.string.disconnect)
        else
            stringResource(id = R.string.connect)
    )
}

@OptIn(ExperimentalPermissionsApi::class)
@Composable
private fun SwitchAudio(mainViewModel: MainViewModel, states: States.UiStates) {

    val permissionsState = rememberMultiplePermissionsState(
        permissions = listOf(Manifest.permission.RECORD_AUDIO)
    )

    Row (
        modifier = Modifier
            .fillMaxWidth()
            .padding(10.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(text = stringResource(id = R.string.turn_audio),
            color = MaterialTheme.colorScheme.onBackground,
            style = MaterialTheme.typography.labelLarge)

        Spacer(Modifier.width(10.dp))

        Switch(
            checked = states.isAudioStarted,
            onCheckedChange = {
                // check for audio permission
                if(!permissionsState.allPermissionsGranted)
                    permissionsState.launchMultiplePermissionRequest()
                else
                    mainViewModel.onEvent(Event.AudioSwitch)

            },
            modifier = Modifier
        )
    }
}