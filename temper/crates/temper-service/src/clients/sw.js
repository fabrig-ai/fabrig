// temper/docs/design/client/web/README.md, 2.7. No fetch handler or cache.
self.addEventListener("push", event => {
    let title = "temper", body = "Something needs you.", project = null;
    try {
        const payload = event.data.json();
        if (typeof payload.title !== "string" || typeof payload.body !== "string" ||
            !Number.isInteger(payload.project) || payload.project < 1 || payload.project > 4294967295) {
            throw new Error("Unreadable push");
        }
        title = payload.title;
        body = payload.body;
        project = payload.project;
    } catch (_) { /* Every push shows a notification, including an unreadable one. */ }
    event.waitUntil(self.registration.showNotification(title, {
        body, tag: `project-${project || 0}`, data: project
    }));
});

self.addEventListener("notificationclick", event => {
    event.notification.close();
    const project = event.notification.data;
    const path = Number.isInteger(project) && project > 0 && project <= 4294967295 ? `/?project=${project}` : "/";
    const url = new URL(path, self.location.origin).href;
    event.waitUntil(self.clients.matchAll({ type: "window", includeUncontrolled: true }).then(async windows => {
        for (const window of windows) {
            if (new URL(window.url).origin === self.location.origin) {
                const navigated = await window.navigate(url);
                if (navigated) return navigated.focus();
            }
        }
        return self.clients.openWindow(url);
    }));
});
