package com.Aspenini.MemoryPak

import android.app.NativeActivity
import android.content.Intent
import android.os.Bundle

class MemoryPakActivity : NativeActivity() {
    private var pendingExport: String? = null

    external fun nativeImportBackup(json: String)
    external fun nativeImportCancelled()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
    }

    fun exportBackup(json: String) = runOnUiThread {
        pendingExport = json
        val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "application/json"
            putExtra(Intent.EXTRA_TITLE, "memory_pak_export.json")
        }
        startActivityForResult(intent, EXPORT_REQUEST)
    }

    fun importBackup() = runOnUiThread {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "application/json"
        }
        startActivityForResult(intent, IMPORT_REQUEST)
    }

    @Deprecated("Activity result bridge for NativeActivity")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        val uri = data?.data
        if (resultCode != RESULT_OK || uri == null) {
            if (requestCode == IMPORT_REQUEST) nativeImportCancelled()
            pendingExport = null
            return
        }
        when (requestCode) {
            EXPORT_REQUEST -> {
                contentResolver.openOutputStream(uri, "wt")?.bufferedWriter().use {
                    it?.write(pendingExport.orEmpty())
                }
                pendingExport = null
            }
            IMPORT_REQUEST -> {
                val json = contentResolver.openInputStream(uri)
                    ?.bufferedReader()
                    ?.use { it.readText() }
                if (json == null) nativeImportCancelled() else nativeImportBackup(json)
            }
        }
    }

    private companion object {
        const val EXPORT_REQUEST = 4101
        const val IMPORT_REQUEST = 4102
    }
}
