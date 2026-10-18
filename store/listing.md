# Microsoft Store listing

Paste into Partner Center → Store listings. Add **English (United States)** and
**Turkish (Türkiye)** as listing languages.

Shared fields:

- **Product name:** Kalk - Squat Breaks (reserved)
- **Privacy policy URL:** https://besinci.github.io/kalk/privacy.html
- **Website:** https://besinci.github.io/kalk/
- **Support contact:** https://github.com/besinci/kalk/issues
- **Category:** Health & fitness
- **Pricing:** Free

## English

### Description

Sitting all day? Kalk (Turkish for "get up!") is a tiny tray app that reminds you to do 10 bodyweight squats after every 45 minutes of active computer use. One minute, no gym, no leaving your desk.

Why squats every 45 minutes? A 2024 study (Scandinavian Journal of Medicine & Science in Sports) found that breaking up 8.5 hours of sitting with 10 squats every 45 minutes lowered the blood-sugar response more than a single 30-minute walk. It was a small study of 18 men, so treat it as a nudge, not a promise.

Kalk only counts real computer time. The timer runs while you use the keyboard or mouse; step away for 10 minutes or put the PC to sleep and it starts over, because you already got up. When it's time, a small popup opens next to the tray icon with a chime. It doesn't steal keyboard focus, so you can finish your sentence first. Tap Done, Snooze or Skip.

An animated character does the reps with you and counts them. Stats show today against your daily goal, your streak, the last 7 days and every squat you've done.

Everything stays on your computer: no account, no cloud, no telemetry, no ads. Kalk only checks how long ago you last touched the keyboard or mouse, never what you typed.

Free and open source under the MIT license.

### Product features

- Reminds you to do 10 squats after every 45 minutes of computer use
- Counts only active time; resets when you step away or the PC sleeps
- Reminder by the tray that doesn't steal keyboard focus
- Done, Snooze or Skip in one click
- Animated character squats along and counts the reps
- Daily goal, streak, last 7 days and total squats
- Active hours and days, plus Pause for 1 hour or for today
- Adjustable interval, reps, goal, snooze, sound and launch at startup
- English and Turkish
- No account, no telemetry, no ads; free and open source

### What's new in this version

First release on the Microsoft Store.

### Short description

A tray app that reminds you to do 10 squats after every 45 minutes at the computer.

### Search terms (max 7, 30 chars each)

squat, break reminder, standing reminder, sitting, desk exercise, health, stretch

### Screenshot captions

1. Click the tray icon for your countdown
2. Squat along when it's time
3. Watch your streak grow
4. Make it fit your day

## Türkçe

### Açıklama

Bütün gün oturuyor musun? Kalk, bilgisayar başında her 45 dakikalık aktif kullanımdan sonra 10 squat yapmanı hatırlatan küçük bir uygulama. Yalnızca bir dakika, spor salonu yok, masandan ayrılmana bile gerek yok.

Neden her 45 dakikada squat? 2024'te yapılan bir çalışma (Scandinavian Journal of Medicine & Science in Sports), 8,5 saatlik oturmayı her 45 dakikada 10 squat ile bölmenin kan şekeri yanıtını tek bir 30 dakikalık yürüyüşten daha fazla düşürdüğünü buldu. 18 erkekle yapılmış küçük bir çalışma; bunu bir vaat değil, bir hatırlatma olarak gör.

Kalk yalnızca gerçek bilgisayar süresini sayar. Sayaç klavyeyi ya da fareyi kullanırken çalışır; 10 dakika uzaklaşırsan ya da bilgisayarı uyku moduna alırsan baştan başlar, çünkü zaten kalkmışsındır. Zamanı gelince ekranın köşesinde bir sesle küçük bir pencere açılır. Klavye odağını çalmaz, cümleni bitirebilirsin. Yaptım, Ertele ya da Atla.

Hareketli bir karakter tekrarları seninle birlikte yapar ve sayar. İstatistikler günlük hedefine göre bugünü, serini, son 7 günü ve bugüne kadar yaptığın tüm squatları gösterir.

Her şey bilgisayarında kalır: hesap yok, bulut yok, telemetri yok, reklam yok. Kalk yalnızca klavyeye ya da fareye en son ne zaman dokunduğuna bakar, ne yazdığına asla.

MIT lisansıyla ücretsiz ve açık kaynak.

### Ürün özellikleri

- Her 45 dakikalık bilgisayar kullanımından sonra 10 squat hatırlatır
- Yalnızca aktif süreyi sayar; uzaklaşınca ya da uyku modunda sıfırlanır
- Ekranın köşesinde açılan, klavye odağını çalmayan hatırlatma
- Tek tıkla Yaptım, Ertele ya da Atla
- Hareketli karakter seninle squat yapar ve tekrarları sayar
- Günlük hedef, seri, son 7 gün ve toplam squat
- Aktif saatler ve günler; 1 saatlik ya da bugünlük duraklatma
- Aralık, tekrar, hedef, erteleme, ses ve açılışta başlatma ayarları
- Türkçe ve İngilizce
- Hesap, telemetri ve reklam yok; ücretsiz ve açık kaynak

### Bu sürümdeki yenilikler

Microsoft Store'daki ilk sürüm.

### Kısa açıklama

Bilgisayar başında her 45 dakikada bir 10 squat yapmanı hatırlatan küçük bir uygulama.

### Arama terimleri (en fazla 7, her biri 30 karakter)

squat, mola hatırlatıcı, hareketsizlik, oturma, masa egzersizi, sağlık, esneme

### Ekran görüntüsü başlıkları

1. Geri sayım için uygulama simgesine tıkla
2. Zamanı gelince birlikte squat yap
3. Serinin büyüdüğünü izle
4. Gününe göre ayarla

## Screenshots

`npm run screenshots` renders them into `store/screenshots/en` and
`store/screenshots/tr` (1920×1080 PNG, the Store's recommended size): the
caption above on the left, the popup with sample data on the right. Upload
them in order:

1. **Info popup** — next break in 23 min, today 5 of 8.
2. **Reminder** — mid-squat at 4/10, Done / Snooze / Skip visible.
3. **Stats** — a 4-day streak and a filled last-7-days chart.
4. **Settings** — the Breaks card.
5. **Reminder, dark theme** — to show it follows the system.

## Partner Center notes

- **runFullTrust justification:** Kalk is a desktop tray app built with Tauri.
  It needs full trust to show a tray icon, read the system idle time
  (GetLastInputInfo) to count only active computer use, and position its
  popup next to the tray. It makes no network connections and collects no data.
- **Age rating (IARC):** no violence, no user interaction or sharing, no
  purchases, no location, no personal data → expect 3+ / Everyone.
- **Store-managed updates:** the in-app updater is off in the MSIX build.
