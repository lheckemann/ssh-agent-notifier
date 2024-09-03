# notifying ssh-agent proxy

Has this ever happened to you?

> You: `git pull`  
> Your YubiKey: *starts blinking to indicate that you should touch it to confirm that SSH should be allowed to authenticate to the git server*  
> You: *waits for git pull to do its thing*  
> Your YubiKey: *times out*  
> git: *fails*  
> You: *is annoyed*

Have no fear, ssh-agent-notifier is here!

Run `ssh-agent-notifier --target unix://$SSH_AUTH_SOCK --host $XDG_RUNTIME_DIR/notifying-agent.sock`, then set `SSH_AUTH_SOCK` to `$XDG_RUNTIME_DIR/notifying-agent.sock` for everything else, and it will notify you whenever something tries to use any keys from your agent!

No warranty, et cetera. This may eat your cats or cause other trouble. In particular, I just ignore errors on notification sending, which means you may not actually get the promised notification and the software won't give a damn. But it works on my computer!
