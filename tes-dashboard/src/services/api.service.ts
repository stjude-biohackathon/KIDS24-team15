import axios from 'axios'

/** defines ApiService object for method access within App logic. Built for modularity if we migrate from axios **/
const ApiService = {

    /** Set BaseURL **/
    init() {
        axios.defaults.baseURL = import.meta.env.VITE_APP_BASE_URL + import.meta.env.VITE_APP_BASE_API;
    },

    /** Accesses Axios headers and adds necessary authorization header **/
    setHeader() {
        axios.defaults.headers.common['Authorization'] = import.meta.env.VITE_APP_AUTH_HEADER
    },

    /** Resets the header to be empty **/
    removeHeader() {
        axios.defaults.headers.common = {}
    },

    /** Standard GET request for a given resource **/
    get(resource: string) {
        return axios.get(resource);
    },

    /** Standard POST request for some data **/
    post(resource: string, data: object) {
        return axios.post(resource, data)
    },

    /** Standard PUT request for some data **/
    put(resource: string, data: object) {
        return axios.put(resource, data)
    },

    /** Standard DELETE request for some resource **/
    delete(resource: string) {
        return axios.delete(resource)
    },

    /** Custom Axios request. The method type is contained as "method" within the data input
     *
     * Perform a custom Axios request.
     *
     * data is an object containing the following properties:
     *  - method
     *  - url
     *  - data ... request payload
     *  - auth (optional)
     *    - username
     *    - password
     **/
    customRequest(data: object) {
        return axios(data)
    }
};

/** Standard export default for APIService to be modular within App logic. If axios is replaced, method access
 * will remain the same **/
export default ApiService
