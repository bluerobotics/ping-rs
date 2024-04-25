(function() {var implementors = {
"tokio":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio/sync/futures/struct.Notified.html\" title=\"struct tokio::sync::futures::Notified\">Notified</a>&lt;'_&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio/sync/oneshot/struct.Receiver.html\" title=\"struct tokio::sync::oneshot::Receiver\">Receiver</a>&lt;T&gt;"]],
"tokio_util":[["impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio_util/sync/struct.WaitForCancellationFuture.html\" title=\"struct tokio_util::sync::WaitForCancellationFuture\">WaitForCancellationFuture</a>&lt;'a&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio_util/sync/struct.ReusableBoxFuture.html\" title=\"struct tokio_util::sync::ReusableBoxFuture\">ReusableBoxFuture</a>&lt;T&gt;"],["impl&lt;L, R, O&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"enum\" href=\"tokio_util/either/enum.Either.html\" title=\"enum tokio_util::either::Either\">Either</a>&lt;L, R&gt;<div class=\"where\">where\n    L: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&lt;Output = O&gt;,\n    R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&lt;Output = O&gt;,</div>"]],
"tracing":[["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tracing/instrument/struct.Instrumented.html\" title=\"struct tracing::instrument::Instrumented\">Instrumented</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tracing/instrument/struct.WithDispatch.html\" title=\"struct tracing::instrument::WithDispatch\">WithDispatch</a>&lt;T&gt;"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()