<template>
  <div class="process-chart">
    <h2>历史趋势</h2>
    <div class="chart-tabs">
      <button
        v-for="proc in processNames"
        :key="proc"
        :class="['tab-btn', { active: selectedProcess === proc }]"
        @click="selectedProcess = proc"
      >
        {{ proc }}
      </button>
    </div>
    <div class="chart-container">
      <Line v-if="chartData" :data="chartData" :options="chartOptions" />
      <div v-else class="no-data">暂无数据</div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, shallowRef } from 'vue'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler
} from 'chart.js'
import { Line } from 'vue-chartjs'

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler
)

const props = defineProps({
  processes: {
    type: Array,
    required: true
  },
  historyData: {
    type: Object,
    required: true
  }
})

const selectedProcess = ref(null)

const processNames = computed(() => {
  return props.processes.map(p => p.name)
})

watch(processNames, (names) => {
  if (names.length > 0 && !selectedProcess.value) {
    selectedProcess.value = names[0]
  }
  if (names.length > 0 && !names.includes(selectedProcess.value)) {
    selectedProcess.value = names[0]
  }
}, { immediate: true })

const isDark = ref(false)

function detectTheme() {
  isDark.value = document.documentElement.classList.contains('dark')
}

onMounted(() => {
  detectTheme()
  const observer = new MutationObserver(() => {
    detectTheme()
  })
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] })
})

const SAMPLE_STEP = 2

const chartData = computed(() => {
  if (!selectedProcess.value) return null
  const rawData = props.historyData[selectedProcess.value]
  if (!rawData || rawData.length === 0) return null

  const sampled = rawData.length > 30
    ? rawData.filter((_, i) => i % SAMPLE_STEP === 0 || i === rawData.length - 1)
    : rawData

  const labels = sampled.map((_, i) => {
    const secondsAgo = (rawData.length - 1 - (rawData.length > 30 ? i * SAMPLE_STEP : i)) * 2
    return secondsAgo <= 0 ? '现在' : `-${secondsAgo}s`
  })

  return {
    labels,
    datasets: [
      {
        label: 'CPU 使用率 (%)',
        data: sampled.map(d => d.cpu_usage || 0),
        borderColor: '#667eea',
        backgroundColor: 'rgba(102, 126, 234, 0.1)',
        yAxisID: 'y',
        tension: 0.3,
        fill: true,
        pointRadius: sampled.length > 20 ? 0 : 3,
        pointHoverRadius: 4,
        borderWidth: 2
      },
      {
        label: '内存占用 (MB)',
        data: sampled.map(d => d.memory_mb || 0),
        borderColor: '#f093fb',
        backgroundColor: 'rgba(240, 147, 251, 0.1)',
        yAxisID: 'y1',
        tension: 0.3,
        fill: true,
        pointRadius: sampled.length > 20 ? 0 : 3,
        pointHoverRadius: 4,
        borderWidth: 2
      }
    ]
  }
})

const chartOptions = computed(() => {
  const textColor = isDark.value ? '#cbd5e1' : '#333333'
  const gridColor = isDark.value ? 'rgba(148, 163, 184, 0.15)' : 'rgba(0,0,0,0.08)'
  return {
    responsive: true,
    maintainAspectRatio: false,
    animation: false,
    interaction: {
      mode: 'index',
      intersect: false,
    },
    plugins: {
      legend: {
        position: 'top',
        labels: {
          color: textColor
        }
      },
      tooltip: {
        backgroundColor: isDark.value ? 'rgba(15, 23, 42, 0.95)' : 'rgba(0, 0, 0, 0.85)',
        titleColor: textColor,
        bodyColor: textColor,
        borderColor: isDark.value ? '#475569' : 'rgba(255,255,255,0.1)',
        borderWidth: 1,
        enabled: true
      }
    },
    scales: {
      x: {
        ticks: { color: textColor },
        grid: { color: gridColor }
      },
      y: {
        type: 'linear',
        display: true,
        position: 'left',
        title: {
          display: true,
          text: 'CPU (%)',
          color: textColor
        },
        min: 0,
        ticks: { color: textColor },
        grid: { color: gridColor }
      },
      y1: {
        type: 'linear',
        display: true,
        position: 'right',
        title: {
          display: true,
          text: '内存 (MB)',
          color: textColor
        },
        min: 0,
        ticks: { color: textColor },
        grid: {
          drawOnChartArea: false,
          color: gridColor
        },
      },
    },
  }
})
</script>

<style scoped>
.process-chart {
  background: var(--bg-secondary);
  border-radius: 12px;
  padding: 20px;
  box-shadow: var(--shadow);
  transition: background 0.3s ease;
}

.process-chart h2 {
  font-size: 18px;
  margin-bottom: 16px;
  color: var(--text-primary);
  transition: color 0.3s ease;
}

.chart-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.tab-btn {
  padding: 6px 16px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-light);
  border-radius: 20px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.tab-btn:hover {
  background: var(--bg-primary);
}

.tab-btn.active {
  background: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
}

.chart-container {
  height: 300px;
  position: relative;
}

.no-data {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
  font-size: 14px;
  transition: color 0.3s ease;
}
</style>
