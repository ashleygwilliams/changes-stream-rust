(function() {var implementors = {};
implementors["tokio_core"] = [];implementors["tokio_tls"] = ["impl&lt;S:&nbsp;<a class='trait' href='tokio_core/io/trait.Io.html' title='tokio_core::io::Io'>Io</a>&gt; <a class='trait' href='tokio_core/io/trait.Io.html' title='tokio_core::io::Io'>Io</a> for <a class='struct' href='tokio_tls/struct.TlsStream.html' title='tokio_tls::TlsStream'>TlsStream</a>&lt;S&gt;",];implementors["hyper"] = ["impl&lt;S&gt; <a class='trait' href='tokio_core/io/trait.Io.html' title='tokio_core::io::Io'>Io</a> for <a class='struct' href='tokio_tls/struct.TlsStream.html' title='tokio_tls::TlsStream'>TlsStream</a>&lt;S&gt; <span class='where'>where S: <a class='trait' href='tokio_core/io/trait.Io.html' title='tokio_core::io::Io'>Io</a></span>",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
