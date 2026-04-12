package com.monochrome.app;

import android.app.Activity;
import android.app.Dialog;
import android.webkit.CookieManager;
import android.webkit.WebResourceRequest;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.webkit.JavascriptInterface;
import android.widget.LinearLayout;
import android.view.ViewGroup;

/**
 * Bridge for YouTube Music login and cookie extraction.
 */
public class YTMBridge {
    private final Activity activity;
    private final WebView mainWebView;

    public YTMBridge(Activity activity, WebView mainWebView) {
        this.activity = activity;
        this.mainWebView = mainWebView;
    }

    @JavascriptInterface
    public void startLogin() {
        activity.runOnUiThread(() -> {
            Dialog dialog = new Dialog(activity, android.R.style.Theme_Black_NoTitleBar_Fullscreen);
            WebView webView = new WebView(activity);
            webView.getSettings().setJavaScriptEnabled(true);
            webView.getSettings().setUserAgentString("Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36");
            
            webView.setWebViewClient(new WebViewClient() {
                @Override
                public void onPageFinished(WebView view, String url) {
                    super.onPageFinished(view, url);
                    if (url.contains("music.youtube.com")) {
                        String cookies = CookieManager.getInstance().getCookie(url);
                        if (cookies != null && cookies.contains("HSID")) { // Basic check for login success
                            extractAndFinish(cookies, dialog);
                        }
                    }
                }
            });

            webView.loadUrl("https://accounts.google.com/ServiceLogin?service=youtube&continue=https://music.youtube.com/");
            dialog.setContentView(webView);
            dialog.show();
        });
    }

    private void extractAndFinish(String cookies, Dialog dialog) {
        activity.runOnUiThread(() -> {
            String js = "window._onYTMLoginSuccess('" + cookies.replace("'", "\\'") + "')";
            mainWebView.evaluateJavascript(js, null);
            dialog.dismiss();
        });
    }
}
