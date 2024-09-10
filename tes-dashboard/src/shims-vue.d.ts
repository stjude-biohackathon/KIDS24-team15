// This code enables TS to recognize imported .vue files as the proper type
// https://stackoverflow.com/questions/54622621/what-does-the-shims-tsx-d-ts-file-do-in-a-vue-typescript-project
declare module "*.vue" {
    import Vue from 'vue';
    export default Vue;
}