package com.gregorycarnegie.ironinsights.data.db

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase
import androidx.sqlite.db.SupportSQLiteDatabase
import com.gregorycarnegie.ironinsights.data.db.dao.*
import com.gregorycarnegie.ironinsights.data.db.entity.*
import com.gregorycarnegie.ironinsights.data.db.seed.DefaultExercises
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

@Database(
    entities = [
        ExerciseDefinition::class,
        WorkoutSession::class,
        ExercisePerformed::class,
        SetEntry::class,
        Programme::class,
        ProgrammeBlock::class,
        PlannedSession::class,
        PlannedExercise::class,
        PlannedSet::class,
        PlateInventory::class,
        PlateInventoryItem::class,
        BarPreset::class,
    ],
    version = 1,
    exportSchema = true,
)
abstract class IronInsightsDatabase : RoomDatabase() {
    abstract fun exerciseDefinitionDao(): ExerciseDefinitionDao
    abstract fun workoutSessionDao(): WorkoutSessionDao
    abstract fun exercisePerformedDao(): ExercisePerformedDao
    abstract fun setEntryDao(): SetEntryDao
    abstract fun trainingStatsDao(): TrainingStatsDao
    abstract fun programmeDao(): ProgrammeDao
    abstract fun plannedSessionDao(): PlannedSessionDao
    abstract fun equipmentDao(): EquipmentDao

    companion object {
        @Volatile
        private var INSTANCE: IronInsightsDatabase? = null

        fun getInstance(context: Context): IronInsightsDatabase {
            return INSTANCE ?: synchronized(this) {
                INSTANCE ?: buildDatabase(context).also { INSTANCE = it }
            }
        }

        private fun buildDatabase(context: Context): IronInsightsDatabase {
            return Room.databaseBuilder(
                context.applicationContext,
                IronInsightsDatabase::class.java,
                "iron_insights_training.db",
            )
                .addCallback(object : Callback() {
                    override fun onCreate(db: SupportSQLiteDatabase) {
                        super.onCreate(db)
                        INSTANCE?.let { database ->
                            CoroutineScope(Dispatchers.IO).launch {
                                val dao = database.exerciseDefinitionDao()
                                if (dao.count() == 0) {
                                    dao.insertAll(DefaultExercises.all)
                                }
                            }
                        }
                    }
                })
                .build()
        }
    }
}
