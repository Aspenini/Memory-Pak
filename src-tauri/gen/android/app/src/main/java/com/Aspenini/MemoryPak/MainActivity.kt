package com.Aspenini.MemoryPak

import android.os.Bundle
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.enableEdgeToEdge
import com.google.android.material.snackbar.Snackbar
import com.google.android.play.core.appupdate.AppUpdateManager
import com.google.android.play.core.appupdate.AppUpdateManagerFactory
import com.google.android.play.core.appupdate.AppUpdateOptions
import com.google.android.play.core.install.InstallStateUpdatedListener
import com.google.android.play.core.install.model.AppUpdateType
import com.google.android.play.core.install.model.InstallStatus
import com.google.android.play.core.install.model.UpdateAvailability

class MainActivity : TauriActivity() {
  private lateinit var appUpdateManager: AppUpdateManager

  private val updateLauncher = registerForActivityResult(
    ActivityResultContracts.StartIntentSenderForResult()
  ) { _ -> }

  private val installListener = InstallStateUpdatedListener { state ->
    if (state.installStatus() == InstallStatus.DOWNLOADED) {
      promptToCompleteStoreUpdate()
    }
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    appUpdateManager = AppUpdateManagerFactory.create(this)
    appUpdateManager.registerListener(installListener)
    checkForStoreUpdate()
  }

  override fun onResume() {
    super.onResume()
    appUpdateManager.appUpdateInfo.addOnSuccessListener { appUpdateInfo ->
      if (appUpdateInfo.installStatus() == InstallStatus.DOWNLOADED) {
        promptToCompleteStoreUpdate()
      }
      if (appUpdateInfo.updateAvailability() ==
        UpdateAvailability.DEVELOPER_TRIGGERED_UPDATE_IN_PROGRESS
      ) {
        appUpdateManager.startUpdateFlowForResult(
          appUpdateInfo,
          updateLauncher,
          AppUpdateOptions.newBuilder(AppUpdateType.FLEXIBLE).build()
        )
      }
    }
  }

  override fun onDestroy() {
    appUpdateManager.unregisterListener(installListener)
    super.onDestroy()
  }

  private fun checkForStoreUpdate() {
    appUpdateManager.appUpdateInfo.addOnSuccessListener { appUpdateInfo ->
      if (
        appUpdateInfo.updateAvailability() == UpdateAvailability.UPDATE_AVAILABLE &&
        appUpdateInfo.isUpdateTypeAllowed(AppUpdateType.FLEXIBLE)
      ) {
        appUpdateManager.startUpdateFlowForResult(
          appUpdateInfo,
          updateLauncher,
          AppUpdateOptions.newBuilder(AppUpdateType.FLEXIBLE).build()
        )
      }
    }
  }

  private fun promptToCompleteStoreUpdate() {
    Snackbar.make(
      findViewById(android.R.id.content),
      getString(R.string.update_ready),
      Snackbar.LENGTH_INDEFINITE
    ).setAction(getString(R.string.update_restart)) {
      appUpdateManager.completeUpdate()
    }.show()
  }
}
