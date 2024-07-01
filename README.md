# Notification Daemon for todayiwill

![Rust](https://img.shields.io/badge/rust-1.79+-green?logo=rust)

Daemon for notifying appointments created with [todayiwill](https://github.com/vncsmyrnk/todayiwill).

## 🔧 Development with docker

```bash
docker run --rm -it \
    -v "$(pwd)":/home/dev/app \
    -v ~/.ssh:/home/dev/.ssh \
    -v /run/user/1000/bus:/run/user/1000/bus \
    -v /tmp/.X11-unix:/tmp/.X11-unix \
    -e DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" \
    -e DISPLAY="$DISPLAY" \
    -e GIT_USERNAME="$(git config --list | grep "user.name" | cut -d = -f2)" \
    -e GIT_EMAIL="$(git config --list | grep "user.email" | cut -d = -f2)" \
    -u dev \
    --cpus 2 \
    --workdir /home/dev/app \
    ghcr.io/vncsmyrnk/rust-dev:latest bash
```

### Dev Tools

Once inside the container, you can run `$ sudo -E ./dev-setup.sh` to install dev dependencies like `git` and `nvim`.
