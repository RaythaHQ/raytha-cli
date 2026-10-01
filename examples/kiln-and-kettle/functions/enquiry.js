/** POST /enquiry {name,email,message,topic}: emails the studio and answers with JSON. */
function post(payload, query) {
    if (!payload || !payload.email || !payload.message) {
        return new StatusCodeResult(400, "email and message are required");
    }
    var esc = function (s) { return String(s || "").replace(/&/g, "&amp;").replace(/</g, "&lt;"); };
    var html = "<p><strong>" + esc(payload.name) + "</strong> (" + esc(payload.email) + ") asked about <em>" + esc(payload.topic || "the studio") + "</em>:</p><p>" + esc(payload.message) + "</p>";
    var sent = true;
    try {
        Emailer.SendEmail(EmailMessage.From("Studio enquiry: " + (payload.topic || "general"), html, "studio@kilnandkettle.example",
            CurrentOrganization.SmtpDefaultFromAddress, CurrentOrganization.OrganizationName));
    } catch (e) {
        sent = false;
    }
    return new JsonResult({ received: true, emailed: sent, message: "Thank you. Someone at the studio will reply within two working days." });
}

function get(query) {
    return new StatusCodeResult(405, "POST an enquiry here");
}
