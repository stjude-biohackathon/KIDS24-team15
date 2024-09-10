/**
 * plugins/vuetify.js
 *
 * Framework documentation: https://vuetifyjs.com`
 */
import {aliases, mdi} from 'vuetify/iconsets/mdi-svg'
import {createVuetify} from 'vuetify'
import {
    mdiWeatherNight,
    mdiWhiteBalanceSunny,
} from '@mdi/js'
import 'vuetify/styles'
export default createVuetify({
    defaults: {
        VAutocomplete: {density: 'compact', variant: 'underlined'},
        VBtn: {variant: 'tonal'},
        VCheckbox: {density: 'compact'},
        VCombobox: {density: 'compact', variant: 'underlined'},
        VSelect: {density: 'compact', variant: 'underlined'},
        VSwitch: {density: 'compact', color: 'primary'},
        VTextField: {density: 'compact', variant: 'underlined'},
    },
    icons: {
        defaultSet: 'mdi',
        aliases: {
            ...aliases,
            weatherNight: mdiWeatherNight,
            whiteBalanceSunny: mdiWhiteBalanceSunny,
        },
        sets: {
            mdi,
        },
    },
    theme: {
        themes: {
            light: {
                colors: {
                    success: '#008a1c',
                    error: '#e03400',
                    primary: '#1874dc',
                    stJudeRedLight: '#d11947',
                    stJudeRedDark: '#8d0034',
                    stJudeGrayLight: '#dfe1df',
                    stJudeGrayDark: '#474c55',
                    stJudeRedLightButton: '#d11947',
                },
            },
            dark: {
                colors: {
                    success: '#008a1c',
                    error: '#e03400',
                    primary: '#1874dc',
                    stJudeRedLightButton: '#d11947',
                }
            }
        },
    },
})
