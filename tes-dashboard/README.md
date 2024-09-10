# Welcome to the TES Dashboard

## Getting Started
This project is a component of the Crankshaft Biohackathon 2024 project. It is used to display the status of jobs submitted to a custom TES environment running on Kubernetes.

To get started,
* Clone the repo
* Run `npm install`
* Add a `local.development.local' file to the root of the project with the following content:
```
VITE_APP_API='INSERT_YOUR_API_URL_HERE'
VITE_APP_AUTH_HEADER='INSERT_YOUR_AUTH_HEADER_HERE'
```
* Run `vite` or `npm run dev` to start the development server

## Developer Notes
This project utilizes Vite, Vue, Vuetify, and one Primevue component (A Pie Chart). Although currently unused, it also contains boilerplate for a Pinia app store, along with Vue-Router, in case future developers might need these functionalities.