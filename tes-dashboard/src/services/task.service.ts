import ApiService from '@/services/api.service.ts'

const dataURL = import.meta.env.VITE_APP_API

class TaskServiceError extends Error {
    errorCode: number

    constructor(errorCode: number, message: string) {
        super(message)
        this.name = this.constructor.name
        this.message = message
        this.errorCode = errorCode
    }
}

const TaskService = {
    listTasks: async function () {
        try {
            const requestData = {
                method: 'get',
                url: dataURL,
            }
            let response = await ApiService.customRequest(requestData)
            return response.data
        } catch (error: any) {
            throw new TaskServiceError(error.response.status, error.response.data)
        }
    },
}

export { TaskService }
