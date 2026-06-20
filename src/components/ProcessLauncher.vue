<template>
  <div class="launcher-panel">
    <div class="panel-header">
      <h2>进程启动器</h2>
      <button class="btn-add" @click="showAddForm = true">
        + 添加启动项
      </button>
    </div>

    <div v-if="showAddForm" class="add-form">
      <h3>{{ editingIndex >= 0 ? '编辑启动项' : '新增启动项' }}</h3>
      <div class="form-grid">
        <div class="form-item">
          <label>显示名称</label>
          <input v-model="form.label" placeholder="如：VS Code" />
        </div>
        <div class="form-item">
          <label>程序路径</label>
          <input v-model="form.path" placeholder="如：C:\\Program Files\\...\\code.exe" />
        </div>
        <div class="form-item">
          <label>启动参数（可选）</label>
          <input v-model="argsStr" placeholder="空格分隔，如：--disable-gpu" />
        </div>
        <div class="form-item">
          <label>监控进程名</label>
          <input v-model="form.process_name" placeholder="如：Code.exe（留空则自动提取文件名）" />
        </div>
        <div class="form-item checkbox-row">
          <label>
            <input type="checkbox" v-model="form.auto_monitor" />
            启动后自动加入监控列表
          </label>
        </div>
      </div>
      <div class="form-actions">
        <button class="btn-cancel" @click="resetForm">取消</button>
        <button class="btn-save" @click="submitForm">保存</button>
      </div>
    </div>

    <div v-if="props.config.launcher && props.config.launcher.length === 0" class="empty-tip">
      暂无启动项，点击右上角"添加启动项"添加
    </div>

    <div v-else class="launcher-grid">
      <div
        v-for="(entry, idx) in props.config.launcher"
        :key="entry.label"
        class="launcher-card"
      >
        <div class="card-icon">🚀</div>
        <div class="card-info">
          <div class="card-label">{{ entry.label }}</div>
          <div class="card-path" :title="entry.path">
            {{ shortPath(entry.path) }}
          </div>
          <div class="card-meta">
            <span v-if="entry.auto_monitor" class="tag auto">自动监控</span>
            <span v-if="entry.args && entry.args.length" class="tag args">含参数</span>
          </div>
        </div>
        <div class="card-actions">
          <button class="btn-launch" @click="handleLaunch(entry.label)" title="启动">
            ▶
          </button>
          <button class="btn-edit" @click="editEntry(idx)" title="编辑">
            ✏️
          </button>
          <button class="btn-delete" @click="handleDelete(entry.label)" title="删除">
            🗑️
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'

const props = defineProps({
  config: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['add-entry', 'remove-entry', 'launch', 'reload'])

const showAddForm = ref(false)
const editingIndex = ref(-1)

const defaultForm = () => ({
  label: '',
  path: '',
  args: null,
  auto_monitor: true,
  process_name: null,
})

const form = ref(defaultForm())

const argsStr = computed({
  get() {
    return form.value.args ? form.value.args.join(' ') : ''
  },
  set(val) {
    const arr = val.trim() ? val.trim().split(/\s+/) : null
    form.value.args = arr
  }
})

function resetForm() {
  form.value = defaultForm()
  editingIndex.value = -1
  showAddForm.value = false
}

function editEntry(idx) {
  const entry = props.config.launcher[idx]
  editingIndex.value = idx
  form.value = JSON.parse(JSON.stringify({
    label: entry.label,
    path: entry.path,
    args: entry.args || null,
    auto_monitor: entry.auto_monitor,
    process_name: entry.process_name || null,
  }))
  showAddForm.value = true
}

function submitForm() {
  if (!form.value.label.trim() || !form.value.path.trim()) {
    alert('名称和路径不能为空')
    return
  }
  const entry = {
    label: form.value.label.trim(),
    path: form.value.path.trim(),
    args: form.value.args && form.value.args.length ? form.value.args : null,
    auto_monitor: !!form.value.auto_monitor,
    process_name: form.value.process_name && form.value.process_name.trim()
      ? form.value.process_name.trim()
      : null,
  }

  if (editingIndex.value >= 0) {
    const oldLabel = props.config.launcher[editingIndex.value].label
    if (oldLabel !== entry.label) {
      emit('remove-entry', oldLabel)
    }
  }
  emit('add-entry', entry)
  resetForm()
}

function handleLaunch(label) {
  emit('launch', label)
}

function handleDelete(label) {
  if (confirm(`确定删除启动项 "${label}"？`)) {
    emit('remove-entry', label)
  }
}

function shortPath(p) {
  if (!p) return ''
  if (p.length <= 50) return p
  return '...' + p.slice(-47)
}
</script>

<style scoped>
.launcher-panel {
  background: var(--bg-secondary);
  border-radius: 12px;
  padding: 20px;
  box-shadow: var(--shadow);
  transition: background 0.3s ease;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

h2 {
  font-size: 18px;
  color: var(--text-primary);
}

h3 {
  font-size: 15px;
  margin-bottom: 12px;
  color: var(--text-primary);
}

.btn-add {
  padding: 8px 18px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: background 0.2s;
}

.btn-add:hover {
  background: var(--primary-hover);
}

.add-form {
  background: var(--bg-tertiary);
  padding: 16px;
  border-radius: 8px;
  margin-bottom: 20px;
}

.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px 16px;
  margin-bottom: 16px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-item label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}

.form-item input[type="text"] {
  padding: 8px 10px;
  border: 1px solid var(--border-color);
  border-radius: 5px;
  font-size: 13px;
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.checkbox-row {
  grid-column: 1 / -1;
  flex-direction: row;
  align-items: center;
  gap: 8px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.btn-cancel {
  padding: 8px 18px;
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
}

.btn-save {
  padding: 8px 18px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
}

.empty-tip {
  text-align: center;
  padding: 40px;
  color: var(--text-muted);
  font-size: 14px;
}

.launcher-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
}

.launcher-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-light);
  border-radius: 10px;
  transition: all 0.2s;
}

.launcher-card:hover {
  border-color: var(--primary-color);
  transform: translateY(-1px);
}

.card-icon {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(102, 126, 234, 0.12);
  border-radius: 10px;
  font-size: 22px;
  flex-shrink: 0;
}

.card-info {
  flex: 1;
  min-width: 0;
}

.card-label {
  font-weight: 600;
  color: var(--text-primary);
  font-size: 15px;
  margin-bottom: 3px;
}

.card-path {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 6px;
  font-family: monospace;
}

.card-meta {
  display: flex;
  gap: 6px;
}

.tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 500;
}

.tag.auto {
  background: rgba(46, 213, 115, 0.15);
  color: var(--success-color);
}

.tag.args {
  background: rgba(102, 126, 234, 0.15);
  color: var(--primary-color);
}

.card-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.card-actions button {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-launch {
  background: var(--success-color);
  color: white;
}

.btn-launch:hover {
  transform: scale(1.08);
  filter: brightness(1.1);
}

.btn-edit {
  background: rgba(102, 126, 234, 0.15);
  color: var(--primary-color);
}

.btn-edit:hover {
  background: rgba(102, 126, 234, 0.3);
}

.btn-delete {
  background: rgba(255, 71, 87, 0.12);
  color: var(--danger-color);
}

.btn-delete:hover {
  background: rgba(255, 71, 87, 0.25);
}
</style>
