package xyz.eremef.tauron_notifier

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.res.Configuration
import android.graphics.Color
import android.widget.RemoteViews
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.io.File
import java.net.HttpURLConnection
import java.net.URL
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone
import org.json.JSONObject

data class WidgetSettings(
    val cityGAID: Long,
    val streetGAID: Long,
    val houseNo: String,
    val streetName: String,
    val theme: String
)

class OutageWidgetProvider : AppWidgetProvider() {

    companion object {
        private const val ACTION_REFRESH = "xyz.eremef.tauron_notifier.ACTION_REFRESH"

        // Light theme colors (from style.css :root)
        private const val LIGHT_PRIMARY = "#D9006C"
        private const val LIGHT_LABEL = "#666666"
        private const val LIGHT_UPDATED = "#999999"

        // Dark theme colors (from style.css [data-theme="dark"])
        private const val DARK_PRIMARY = "#FF4DA6"
        private const val DARK_LABEL = "#A0A0A0"
        private const val DARK_UPDATED = "#777777"
    }

    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == ACTION_REFRESH) {
            val mgr = AppWidgetManager.getInstance(context)
            val ids = mgr.getAppWidgetIds(
                ComponentName(context, OutageWidgetProvider::class.java)
            )
            onUpdate(context, mgr, ids)
        }
        super.onReceive(context, intent)
    }

    override fun onUpdate(
        context: Context,
        appWidgetManager: AppWidgetManager,
        appWidgetIds: IntArray
    ) {
        val pendingResult = goAsync()
        CoroutineScope(Dispatchers.IO).launch {
            try {
                for (appWidgetId in appWidgetIds) {
                    updateWidget(context, appWidgetManager, appWidgetId)
                }
            } finally {
                pendingResult.finish()
            }
        }
    }

    private fun loadSettings(context: Context): WidgetSettings? {
        // Try filesDir first (Tauri app_data_dir usually maps here)
        var settingsFile = File(context.filesDir, "settings.json")
        if (!settingsFile.exists()) {
            // Fallback to dataDir
            settingsFile = File(context.dataDir, "settings.json")
        }
        if (!settingsFile.exists()) return null

        return try {
            val json = JSONObject(settingsFile.readText())
            WidgetSettings(
                cityGAID = json.getLong("cityGAID"),
                streetGAID = json.getLong("streetGAID"),
                houseNo = json.getString("houseNo"),
                streetName = json.getString("streetName"),
                theme = json.optString("theme", "system")
            )
        } catch (e: Exception) {
            null
        }
    }

    private fun isDarkMode(context: Context, themeSetting: String): Boolean {
        return when (themeSetting) {
            "dark" -> true
            "light" -> false
            else -> {
                // "system" or missing — follow Android system setting
                val nightMode = context.resources.configuration.uiMode and
                        Configuration.UI_MODE_NIGHT_MASK
                nightMode == Configuration.UI_MODE_NIGHT_YES
            }
        }
    }

    private fun applyTheme(views: RemoteViews, dark: Boolean) {
        if (dark) {
            views.setInt(R.id.widget_root, "setBackgroundResource", R.drawable.widget_background_dark)
            views.setTextColor(R.id.widget_count, Color.parseColor(DARK_PRIMARY))
            views.setTextColor(R.id.widget_label, Color.parseColor(DARK_LABEL))
            views.setTextColor(R.id.widget_updated, Color.parseColor(DARK_UPDATED))
        } else {
            views.setInt(R.id.widget_root, "setBackgroundResource", R.drawable.widget_background)
            views.setTextColor(R.id.widget_count, Color.parseColor(LIGHT_PRIMARY))
            views.setTextColor(R.id.widget_label, Color.parseColor(LIGHT_LABEL))
            views.setTextColor(R.id.widget_updated, Color.parseColor(LIGHT_UPDATED))
        }
    }

    private fun updateWidget(
        context: Context,
        appWidgetManager: AppWidgetManager,
        appWidgetId: Int
    ) {
        val views = RemoteViews(context.packageName, R.layout.widget_outage)

        // Set tap-to-refresh intent
        val refreshIntent = Intent(context, OutageWidgetProvider::class.java).apply {
            action = ACTION_REFRESH
        }
        val refreshPending = PendingIntent.getBroadcast(
            context, 0, refreshIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        views.setOnClickPendingIntent(R.id.widget_root, refreshPending)

        // Load settings
        val settings = loadSettings(context)

        // Apply theme (even before settings are fully loaded)
        val dark = isDarkMode(context, settings?.theme ?: "system")
        applyTheme(views, dark)

        if (settings == null) {
            views.setTextViewText(R.id.widget_count, "?")
            views.setTextViewText(R.id.widget_updated, "Setup needed")
            appWidgetManager.updateAppWidget(appWidgetId, views)
            return
        }

        // Show loading state
        views.setTextViewText(R.id.widget_count, "…")
        views.setTextViewText(R.id.widget_updated, "Updating...")
        appWidgetManager.updateAppWidget(appWidgetId, views)

        // Fetch data
        try {
            val count = fetchFilteredOutageCount(settings)
            val timeFormat = SimpleDateFormat("HH:mm", Locale.getDefault())
            val updatedAt = timeFormat.format(Date())

            views.setTextViewText(R.id.widget_count, count.toString())
            views.setTextViewText(R.id.widget_updated, updatedAt)
        } catch (e: Exception) {
            views.setTextViewText(R.id.widget_count, "!")
            val errMsg = (e.message ?: "Unknown").take(20)
            views.setTextViewText(R.id.widget_updated, errMsg)
        }
        appWidgetManager.updateAppWidget(appWidgetId, views)
    }

    private fun fetchFilteredOutageCount(settings: WidgetSettings): Int {
        val dateFormat = SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'", Locale.US)
        dateFormat.timeZone = TimeZone.getTimeZone("UTC")
        val now = dateFormat.format(Date())
        val baseUrl = "https://www.tauron-dystrybucja.pl/waapi/outages/address"
        val params = "cityGAID=${settings.cityGAID}&streetGAID=${settings.streetGAID}" +
                "&houseNo=${settings.houseNo}" +
                "&fromDate=$now&getLightingSupport=false" +
                "&getServicedSwitchingoff=true&_=${System.currentTimeMillis()}"

        val url = URL("$baseUrl?$params")
        val conn = url.openConnection() as HttpURLConnection
        conn.requestMethod = "GET"
        conn.setRequestProperty("accept", "application/json")
        conn.setRequestProperty("x-requested-with", "XMLHttpRequest")
        conn.setRequestProperty("Referer", "https://www.tauron-dystrybucja.pl/wylaczenia")
        conn.connectTimeout = 10000
        conn.readTimeout = 10000

        val responseCode = conn.responseCode
        if (responseCode !in 200..299) {
            conn.disconnect()
            throw Exception("HTTP error: $responseCode")
        }

        val response = conn.inputStream.bufferedReader().readText()
        conn.disconnect()

        val json = JSONObject(response)
        val items = json.optJSONArray("OutageItems") ?: return 0

        var count = 0
        for (i in 0 until items.length()) {
            val item = items.getJSONObject(i)
            val message = item.optString("Message", "")
            if (message.contains(settings.streetName)) {
                count++
            }
        }
        return count
    }
}
