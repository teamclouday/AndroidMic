package io.github.teamclouday.AndroidMic.ui.home

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import io.github.teamclouday.AndroidMic.ui.MainViewModel
import io.github.teamclouday.AndroidMic.ui.components.ManagerButton
import io.github.teamclouday.AndroidMic.ui.home.dialog.BaseDialog

fun getWifiPermission(): MutableList<String> {
    val list = mutableListOf<String>()

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU)
        list.add(Manifest.permission.POST_NOTIFICATIONS)

    return list
}

fun getBluetoothPermission(): MutableList<String> {
    val list = mutableListOf<String>()

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU)
        list.add(Manifest.permission.POST_NOTIFICATIONS)

    list.add(Manifest.permission.BLUETOOTH)

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S)
        list.add(Manifest.permission.BLUETOOTH_CONNECT)

    return list
}

fun getUsbPermission(): MutableList<String> {
    val list = mutableListOf<String>()

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU)
        list.add(Manifest.permission.POST_NOTIFICATIONS)

    return list
}

fun getRecordAudioPermission(): MutableList<String> {
    val list = mutableListOf<String>()

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU)
        list.add(Manifest.permission.POST_NOTIFICATIONS)

    list.add(Manifest.permission.RECORD_AUDIO)

    return list
}

fun Activity.openAppSettings() {
    Intent(
        Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
        Uri.fromParts("package", packageName, null)
    ).also(::startActivity)
}

@Composable
fun PermissionDialog(
    vm: MainViewModel,
    expanded: MutableState<Boolean>,
    onRequestPermissionAgain: () -> Unit,
    onOpenPermissionSetting: () -> Unit
) {

    BaseDialog(
        expanded
    ) {

        Text("We need the requested permission for the app to function properly")

        ManagerButton(
            text = "Request permissions again",
            onClick = {
                onRequestPermissionAgain()
                expanded.value = false
            }
        )

        ManagerButton(
            text = "Allow permissions manually",
            onClick = {
                onOpenPermissionSetting()
                expanded.value = false
            }
        )

    }
}