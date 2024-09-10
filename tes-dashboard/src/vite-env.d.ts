/// <reference types="vite/client" />
/// <reference types="vite/client" />

interface ImportMetaEnv {
    readonly VITE_APP_BASE_URL: string
    readonly VITE_APP_BASE_API: string
    readonly VITE_APP_LOGIN_URL: string
    readonly VITE_APP_LOGOUT_URL: string
}

interface ImportMeta {
    readonly env: ImportMetaEnv
}
