import {useAppStore} from '@/store/app.ts'

const store = useAppStore()

export interface SnackBarData {
    color: string,
    on: boolean,
    text: string,
}

const ErrorService = {
    activateSnackBar: function (d: SnackBarData, snackData: SnackBarData) {
        snackData.on = true
        snackData.color = d.color
        snackData.text = d.text
    },
    handleError: async function (message: string, snackData: SnackBarData, error: any) {
        if (error.errorCode && error.errorCode === 401) {
            await store.logout()
        }
        ErrorService.activateSnackBar({
            color: 'error',
            text: error?.message ? message + ': ' + JSON.stringify(error.message).slice(0, 750) + '...' : message,
            on: true,
        }, snackData)
    },
    handleSuccess: function (message: string, snackData: SnackBarData) {
        ErrorService.activateSnackBar({
            color: 'success',
            text: message,
            on: true,
        }, snackData)
    },
    handleWarning: function (message: string, snackData: SnackBarData) {
        ErrorService.activateSnackBar({
            color: 'warning',
            text: message,
            on: true,
        }, snackData)
    },
    resetSnackBar: (snackData: SnackBarData) => {
        snackData.on = false
        snackData.color = ''
        snackData.text = ''
    }
}

export {ErrorService}
