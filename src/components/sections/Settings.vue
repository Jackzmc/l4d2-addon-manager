<template>
<div class="container">
    <div class="box" v-if="hasChanged">
        <p>You have unsaved changes</p>
        <b-button type="is-success">Save</b-button>
    </div>
    <div class="mx-5 mt-4 box">
        <div class="field">
            <div class="field is-horizontal">
                <div class="field-label is-normal">
                    <label class="label">Addons Folder</label>
                </div>
                <div class="field-body">
                    <div class="field has-addons has-addons-centered">
                        <p class="control">
                            <input class="input" style="width:600px" type="text" :value="settings.gamedir">
                        </p>
                        <!-- TODO: Implement prompting for gamedir -->
                        <p class="control" >
                            <a class="button is-info">
                            Browse
                            </a>
                        </p>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <div class="mx-5 mt-4 box">
        <div class="field">
            <div class="field is-horizontal">
                <div class="field-label is-normal">
                    <label class="label">Telemetry</label>
                </div>
                <div class="field-body">
                    <div class="field has-addons has-addons-centered">
                        <b-switch v-model="changed.telemetry">
                            {{ changed.telemetry ? 'Enabled' : 'Disabled' }}
                        </b-switch>
                    </div>
                </div>
            </div>
            <p class="help is-info">The following information is sent: program version, OS Type, CPU Architecture, and the count of all your managed items</p>
        </div>
    </div>
</div>
</template>

<script>


export default {
    props: ['settings'],
    data() {
        return {
            changed: {
                telemetry: null
            }
        }
    },
    computed: {
        hasChanged() {
            for(const item in this.settings) {
                if(this.settings[item] != this.changed[item]) return true
            }
            return false
        }
    },
    methods: {
        async save() {
            try {
                await invoke('save_settings', { changed: this.changed })
            } catch(err) {
                console.log(err)
            }
        }
    },
    created() {
        this.changed = Object.assign({}, this.settings)
    }
}
</script>

<style scoped>
li {
    margin-bottom: 1em;
}
</style>