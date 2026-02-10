package xyz.eremef.tauron_notifier

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.Context
import android.content.Intent
import android.widget.RemoteViews
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.net.HttpURLConnection
import java.net.URL
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone
import org.json.JSONObject

class OutageWidgetProvider : AppWidgetProvider() {

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

    private fun updateWidget(
        context: Context,
        appWidgetManager: AppWidgetManager,
        appWidgetId: Int
    ) {
        val views = RemoteViews(context.packageName, R.layout.widget_outage)

        // Set tap-to-open intent
        val intent = Intent(context, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            context, 0, intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        views.setOnClickPendingIntent(R.id.widget_root, pendingIntent)

        // Show loading state
        views.setTextViewText(R.id.widget_count, "…")
        views.setTextViewText(R.id.widget_updated, "Updating...")
        appWidgetManager.updateAppWidget(appWidgetId, views)

        // Fetch data
        try {
            val count = fetchFilteredOutageCount()
            val timeFormat = SimpleDateFormat("HH:mm", Locale.getDefault())
            val updatedAt = timeFormat.format(Date())

            views.setTextViewText(R.id.widget_count, count.toString())
            views.setTextViewText(R.id.widget_updated, updatedAt)
        } catch (e: Exception) {
            views.setTextViewText(R.id.widget_count, "!")
            views.setTextViewText(R.id.widget_updated, "Error")
        }
        appWidgetManager.updateAppWidget(appWidgetId, views)
    }

    private fun fetchFilteredOutageCount(): Int {
        val dateFormat = SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'", Locale.US)
        dateFormat.timeZone = TimeZone.getTimeZone("UTC")
        val now = dateFormat.format(Date())
        val baseUrl = "https://www.tauron-dystrybucja.pl/waapi/outages/address"
        val params = "cityGAID=119431&streetGAID=897300&houseNo=8" +
                "&fromDate=$now&getLightingSupport=true" +
                "&getServicedSwitchingoff=true&_=${System.currentTimeMillis()}"

        val url = URL("$baseUrl?$params")
        val conn = url.openConnection() as HttpURLConnection
        conn.requestMethod = "GET"
        conn.setRequestProperty("accept", "application/json")
        conn.setRequestProperty("x-requested-with", "XMLHttpRequest")
        conn.setRequestProperty("Referer", "https://www.tauron-dystrybucja.pl/wylaczenia")
        conn.connectTimeout = 15000
        conn.readTimeout = 15000

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
            if (message.contains("Rozbrat")) {
                count++
            }
        }
        return count
    }
}
