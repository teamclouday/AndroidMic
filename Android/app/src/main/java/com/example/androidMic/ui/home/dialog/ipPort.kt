package com.example.androidMic.ui.home.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.example.androidMic.DefaultStates
import com.example.androidMic.R
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.components.ManagerButton
import com.example.androidMic.ui.components.ManagerOutlinedTextField

@Composable
fun DialogIpPort(vm: MainViewModel, expanded: MutableState<Boolean>) {

    val tempIp = rememberSaveable {
        mutableStateOf(vm.prefs.ip.getBlocking())
    }
    val tempPort = rememberSaveable {
        mutableStateOf(vm.prefs.port.getBlocking())
    }

    BaseDialog(
        expanded,
    ) {
        Column(
            horizontalAlignment = Alignment.End
        ) {
            // reset button
            ManagerButton(
                onClick = {
                    tempIp.value = DefaultStates.IP; tempPort.value = DefaultStates.PORT
                },
                text = stringResource(id = R.string.reset),
                modifier = Modifier.padding(end = 10.dp),
            )

            DialogSpacer()
            Column(
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                // ip field
                ManagerOutlinedTextField(tempIp, R.string.dialog_ip)

                Spacer(modifier = Modifier.height(10.dp))

                // port field
                ManagerOutlinedTextField(tempPort, R.string.dialog_port)

                Spacer(modifier = Modifier.height(20.dp))

                // save Button
                ManagerButton(
                    onClick = {
                        vm.setIpPort(tempIp.value, tempPort.value)
                        expanded.value = false
                    },
                    text = stringResource(id = R.string.save),
                    modifier = Modifier.fillMaxWidth(0.6f)
                )

                Spacer(modifier = Modifier.height(10.dp))

                // cancel Button
                ManagerButton(
                    onClick = {
                        expanded.value = false
                    },
                    text = stringResource(id = R.string.cancel),
                    modifier = Modifier.fillMaxWidth(0.6f)
                )
            }
        }
    }
}
